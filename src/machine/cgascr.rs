const vga_buffer:*mut u8 = 0xb8000 as *mut u8;

#[allow(dead_code)]
pub struct CGAScreen{
    pub max_cows:u32,
    pub max_rows:u32,
}

#[allow(dead_code)]
impl CGAScreen{
    pub fn new(cows:u32, rows:u32) -> Self {
        Self {max_cows: cows, max_rows:rows,}
    }

    
    // this function should be the only one that "directly touches"
    // the memory by address. 
    // and since it's unsafe, it shouldn't be public
    //
    // fn show(&self, x:u32, y:u32, c:char, attr:u8){
    //
    // }
    pub fn test(&self){
        let mut r = 0;
        let mut c = 0;

        while r<self.max_rows {
            while c<self.max_cows {
                let index:u32 = r*self.max_cows + c;
                unsafe {
                    *vga_buffer.offset(index as isize * 2) = index as u8;
                    *vga_buffer.offset(index as isize * 2 + 1) = index as u8;
                }
                c+=1;
            }
            r+=1;
            c=0;
        }


    }

}
