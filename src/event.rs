#[derive(PartialEq, Debug)]
pub enum Event {
    End = 0,
    Data = 1,
    Disconnect = 2,
    Set = 3,
    Stream = 4,
    Ok = 5,
    Status = 6,
    Len = 7,
}
