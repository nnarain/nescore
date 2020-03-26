//
// mapper/mem.rs
//
// @author Natesh Narain <nnaraindev@gmail.com>
// @date Dec 27 2019
//

/// Representation of a memory block in the mapper
pub struct Memory {
    mem: Vec<u8>,
    num_banks: usize,
    bank_size: usize,
}

impl Memory {
    pub fn new(mem: Vec<u8>, bank_size: usize) -> Self {
        let num_banks = mem.len() / bank_size;
        Memory {
            mem,
            num_banks,
            bank_size,
        }
    }

    /// Read from memory
    pub fn read(&self, bank: usize, index: usize) -> u8 {
        let bank_offset = self.get_bank_offset(bank);
        self.mem[bank_offset + index]
    }

    pub fn write(&mut self, bank: usize, index: usize, value: u8) {
        let bank_offset = self.get_bank_offset(bank);
        self.mem[bank_offset + index] = value;
    }

    /// Explicity read from the first bank of memory
    pub fn read_first(&self, index: usize) -> u8 {
        self.read(0, index)
    }

    /// Explicitly read from the last bank of memory
    pub fn read_last(&self, index: usize) -> u8 {
        self.read(self.num_banks - 1, index)
    }

    pub fn set_bank_size(&mut self, new_size: usize) {
        self.bank_size = new_size;
        self.num_banks = self.mem.len() / self.bank_size;
    }

    fn get_bank_offset(&self, bank_num: usize) -> usize {
        bank_num * self.bank_size
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_bank() {
        let mut data = vec![0; 10];
        data[5] = 0xDE;

        let mem = Memory::new(data, 10);

        assert_eq!(mem.read(0, 5), 0xDE);
    }

    #[test]
    #[should_panic]
    fn read_bank_out_of_range() {
        let mut data = vec![0; 10];
        data[5] = 0xDE;

        let mem = Memory::new(data, 10);

        assert_eq!(mem.read(1, 5), 0xDE);
    }

    #[test]
    fn read_bank_2() {
        let mut data = vec![0; 20];
        data[10] = 0xDE;

        let mem = Memory::new(data, 10);

        assert_eq!(mem.read(1, 0), 0xDE);
    }
}
