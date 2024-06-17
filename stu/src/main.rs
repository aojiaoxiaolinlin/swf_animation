[derive(Debug)]
struct Data {
    data: Vec<u8>,
}

fn main() {
    let data = Data { data: vec![1, 2, 3] };
    let data2 = data;
    println!("{:?}", data2.data);
}
