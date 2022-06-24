
// fn i32_to_chinese_writing_style(n: i32) -> String {
//     fn number_to_chinese_character(c: char) -> char {
//         match c {
//             '0' => '零',
//             '1' => '一',
//             '2' => '二',
//             '3' => '三',
//             '4' => '四',
//             '5' => '五',
//             '6' => '六',
//             '7' => '七',
//             '8' => '八',
//             '9' => '九',
//             _ => panic!("{}", "no allow character")
//         }
//     }
//
//     fn place_to_chinese_character(c: usize) -> Option<char> {
//         const 个位: usize = 1;
//         const 十位: usize = 2;
//         const 百位: usize = 3;
//         const 千位: usize = 4;
//         const 万位: usize = 5;
//         const 亿位: usize = 9;
//         match c {
//             个位 => None,
//             十位 => '十',
//             百位 => '百',
//             千位 => '千',
//             万位 => '万',
//             // x @ 万位..亿位  if x != 万位 => None,
//             亿位 => '亿',
//             _ => panic!("{}", "no allow character"),
//         }
//     }
//
//     let s = n.to_string();
//
//
//     s
// }