const CGA_START:u32 = 0xb8000;

#[allow(dead_code)]
pub struct CGAScreen{
    max_cows:u32,
    max_rows:u32,
}

#[allow(dead_code)]
impl CGAScreen{
    pub fn new(cows:u32, rows:u32) -> Self {
        Self {max_cows: cows, max_rows:rows,}
    }

    pub fn set_pos(x:u32, y:u32){

    }

    pub fn get_pos(x:&mut u32, y:&mut u32){
        // TODO
        *x = 1;
        *y = 1;
    }

    pub fn putchar(c:char, attr:u8){

    }
    
    // this function should be the only one that "directly touches"
    // the memory by address. 
    // and since it's unsafe, it shouldn't be public
    fn show(&self, x:u32, y:u32, c:char, attr:u8){

    }

}
