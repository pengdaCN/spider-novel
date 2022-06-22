use spider_novel::qubige::novel;

#[test]
fn get_sort() {
    use spider_novel::qubige::sort;
    let sorts = tokio_test::block_on(sort::get_sort()).unwrap();
    for sort in &sorts {
        println!("{:?}", sort);
    }
}

#[test]
fn get_intro_novel() {
    use spider_novel::qubige::novel;
    let novels = tokio_test::block_on(novel::from_sort(
        "https://www.qubige.com/sort/xiandaixiaoshuo/",
        novel::GetOpt::First,
    )).unwrap();

    for x in &novels {
        println!("{:?}", x);
    }
}