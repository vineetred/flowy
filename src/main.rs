
fn main() {
    let  path = "file:///home/vineet/Desktop/69561.jpg";
    // println!("{}",path);
    // paper_switch::set_paper(path);
    let op = paper_switch::get_envt().unwrap();
    // println!("{:?}", paper_switch::get_envt());
    println!("{}",op)

}
