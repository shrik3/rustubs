const vga_buffer:*mut u8 = 0xb8000 as *mut u8;

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

   
    fn get_index(&self,row:u32, col:u32) -> u32{
        col + row*self.max_cows
    }

    // this function should be the only one that "directly touches"
    // the memory by address. 
    // and since it's unsafe, it shouldn't be public
    pub fn show(&self, row:u32, col:u32, c:char, attr:u8){
        let index = self.get_index(row, col);

        unsafe{
            *vga_buffer.offset(index as isize * 2) = c as u8;
            *vga_buffer.offset(index as isize * 2 + 1) = attr;
        }

    }

    pub fn putchar(&self, ch:char){
         
    }
    
    pub fn setpos(){}
    pub fn getpos(){}
     

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
