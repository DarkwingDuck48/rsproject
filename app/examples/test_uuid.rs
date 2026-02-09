use std::vec;

fn main() {
    let mut vec1 = vec![
        uuid::Uuid::new_v4(),
        uuid::Uuid::new_v4(),
        uuid::Uuid::new_v4(),
        uuid::Uuid::new_v4(),
    ];

    let some_uuid = vec1[0];
    // let some_UUID = uuid::Uuid::new_v4();
    println!("{:?}", vec1);
    vec1.retain(|&x| x != some_uuid);

    println!("{:?}", vec1)
}
