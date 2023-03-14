use crate::arch::x86_64::io_port::*;

const CGA_BUFFER_START:*mut u8 = 0xb8000 as *mut u8;
const IR_PORT:u16 = 0x3d4;
const DR_PORT:u16 = 0x3d5;

#[allow(dead_code)]
pub struct CGAScreen{
    pub max_cows:u32,
    pub max_rows:u32,
}

#[allow(dead_code)]
impl CGAScreen{
    pub fn new(rows:u32, cols:u32) -> Self {
        Self {max_rows:rows, max_cows:cols}
    }

   
    fn cal_offset(&self,row:u32, col:u32) -> u32{
        col + row*self.max_cows
    }

    // this function should be the only one that "directly touches"
    // the memory by address. 
    // and since it's unsafe, it shouldn't be public
    pub fn show(&self, row:u32, col:u32, c:char, attr:u8){
        let index = self.cal_offset(row, col);

        unsafe{
            *CGA_BUFFER_START.offset(index as isize * 2) = c as u8;
            *CGA_BUFFER_START.offset(index as isize * 2 + 1) = attr;
        }

    }

    // pub fn putchar(&self, ch:char){
    //      
    // }
    
    pub fn setpos(&self, row:u32, col:u32){
        // io ports for instruction register and data register
        let offset = self.cal_offset(row,col);
        
        // set lower byte
        outb(IR_PORT, 15 as u8);
        outb(DR_PORT, offset as u8);
        // set higher byte
        outb(IR_PORT, 14 as u8);
        outb(DR_PORT, (offset >> 8) as u8);

    }

    pub fn getoffset() -> u32 {
        // read higher byte
        outb(IR_PORT, 14 as u8);
        let mut offset = inb(DR_PORT);
        offset = offset << 8;
        // read lower byte
        outb(IR_PORT, 15 as u8);
        offset += inb(DR_PORT);
        offset as u32
    }

    pub fn getpos(&self, row:& mut u32, cow:& mut u32){
        let offset = Self::getoffset();
        *row = offset % self.max_cows;
        *cow = offset / self.max_cows;
    }
     

    // Sanity Check of the cgascreen
    pub fn test(&self){
        let mut r = 0;
        let mut c = 0;
    
        let mut counter = 0;
        while r<self.max_rows {
            while c<self.max_cows {
                let ch = (counter & 0xff) as u8;
                self.show(r,c,ch as char, ch);
                counter += 1;
                c+=1;
            }
            r+=1;
            c=0;
        }


    }

}
