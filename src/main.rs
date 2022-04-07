use ckb_vm::registers::{A0, A1, A7};
use ckb_vm::{Memory, Register, SupportMachine, Syscalls};

pub struct SyscallLoad {
    data: Vec<u8>,
}

impl<Mac: SupportMachine> Syscalls<Mac> for SyscallLoad {
    fn initialize(&mut self, _machine: &mut Mac) -> Result<(), ckb_vm::error::Error> {
        Ok(())
    }

    fn ecall(&mut self, machine: &mut Mac) -> Result<bool, ckb_vm::error::Error> {
        let code = &machine.registers()[A7];
        if code.to_i32() != 2077 {
            return Ok(false);
        }

        let data_addr = machine.registers()[A0].clone();
        let size_addr = machine.registers()[A1].clone();
        let size = machine.memory_mut().load64(&size_addr)?.to_u64() as usize;
        let size_real = if size > self.data.len() {
            self.data.len()
        } else {
            size
        };

        machine
            .memory_mut()
            .store_bytes(data_addr.to_u64(), &self.data[0..size_real])?;
        machine
            .memory_mut()
            .store64(&size_addr, &Mac::REG::from_u64(size_real as u64))?;

        Ok(true)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let code = std::fs::read(&args[1])?.into();

    let mut aot_machine = ckb_vm::machine::aot::AotCompilingMachine::load(
        &code,
        Some(Box::new(|_| 1)),
        ckb_vm::ISA_IMC | ckb_vm::ISA_B | ckb_vm::ISA_MOP,
        ckb_vm::machine::VERSION1,
    )?;
    let aot_code = aot_machine.compile()?;

    let asm_core = ckb_vm::machine::asm::AsmCoreMachine::new(
        ckb_vm::ISA_IMC | ckb_vm::ISA_B | ckb_vm::ISA_MOP,
        ckb_vm::machine::VERSION1,
        u64::MAX,
    );
    let core =
        ckb_vm::DefaultMachineBuilder::<Box<ckb_vm::machine::asm::AsmCoreMachine>>::new(asm_core)
            .instruction_cycle_func(Box::new(|_| 1))
            .syscall(Box::new(SyscallLoad {
                data: vec![0x01, 0x02, 0x03, 0x04],
            }))
            .build();
    let mut machine = ckb_vm::machine::asm::AsmMachine::new(core, Some(&aot_code));

    machine.load_program(&code, &vec!["main".into()])?;

    let exit = machine.run();
    let cycles = machine.machine.cycles();
    println!("aot exit={:?} cycles={:?}", exit, cycles,);
    std::process::exit(exit.unwrap() as i32);
}
