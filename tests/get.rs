use fast_log::Config;
use log::info;

use spider_novel::qubige::novel;
use spider_novel::qubige::novel::section;
use spider_novel::spider::{Novel, NovelID};

#[test]
fn get_sort() {
    use spider_novel::qubige::sort;
    fast_log::init(Config::new().console()).unwrap();

    let sorts = tokio_test::block_on(sort::get_sort()).unwrap();
    for sort in &sorts {
        info!("{:?}", sort);
    }
}

#[test]
fn get_intro_novel() {
    use spider_novel::qubige::novel;
    fast_log::init(Config::new().console()).unwrap();
    let novels = tokio_test::block_on(novel::from_sort(
        "https://www.qubige.com/sort/xiandaixiaoshuo/",
        novel::GetOpt::First,
    )).unwrap();

    for x in &novels {
        info!("{:?}", x);
    }
}

#[test]
fn get_novel_intro() {
    use spider_novel::qubige::novel;
    fast_log::init(Config::new().console()).unwrap();

    let n = novel::Novel::new("火车".to_string(), "宫部美雪".to_string(), None, "/booke/e29045b2/".to_string());

    let intro = tokio_test::block_on(n.intro()).unwrap();
    if let Some(v) = intro {
        info!("intro = {}", v);
    }
}

#[test]
fn get_novel_section() {
    use spider_novel::qubige::novel;
    fast_log::init(Config::new().console()).unwrap();

    let n = novel::Novel::new("火车".to_string(), "宫部美雪".to_string(), None, "/booke/e29045b2/".to_string());

    let section = tokio_test::block_on(n.sections()).unwrap();
    if let Some(sections) = section {
        for x in &sections {
            info!("{:?}", x);
        }
    }
}

#[test]
fn get_section_contents() {
    use spider_novel::qubige::novel::section;
    fast_log::init(Config::new().console()).unwrap();

    let section = section::Section::new("第一章".to_string(), "/bookb/b626dbef/894c54902066.html".to_string());

    let contents = tokio_test::block_on(section.contents()).unwrap();
    if let Some(v) = contents {
        for x in &v {
            info!("{}", x);
        }
    }
}

#[test]
fn into() {
    let n: NovelID = (10 as i64).into();

    NovelID::from(10 as i64);

    let n2: i64 = n.into();

    println!("{} {}", n, n2);

    let x = Novel{
        id: 1.into(),
        name: "".to_string(),
        cover: None,
        author: "".to_string(),
        last_updated_at: None,
        last_updated_section_name: None
    };
}

#[test]
fn get_snowflake() {
    use snowflake::SnowflakeIdGenerator;

    let mut gen = SnowflakeIdGenerator::new(1,1);
    for _ in 0..100 {
        println!("{}", gen.generate())
    }
}