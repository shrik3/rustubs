
extern "C" {
    fn inb(port:u32) -> u32;
    fn inw(port:u32) -> u32;
    fn outb(port:u32, val:u32);
    fn outw(port:u32, val:u32);
}

// TODO
// pub struct IO_Port {
//     addr: u32,
// }

