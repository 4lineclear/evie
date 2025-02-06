// use ropey::Rope;
//
// use crate::{
//     buffer::Buffer,
//     engine::{Delete, Edit, Insert, Replace},
// };
//
// macro_rules! check {
//     ($src:expr, $ex:expr $(, $edit:expr, $edit_res:expr)*) => {
//         #[allow(unused_mut)]
//         let mut file = Buffer {
//             text: Rope::from_str($src),
//             ..Buffer::default()
//         };
//         $(
//             let act = file.edit($edit);
//             let res: ropey::Result<()> = $edit_res;
//             assert_eq!(format!("{act:#?}"), format!("{res:#?}"));
//         )*
//         assert_eq!(file.text, $ex);
//     };
// }
//
// #[test]
// fn empty() {
//     check!("", "");
// }
//
// #[test]
// fn yeah() {
//     check!("yeah", "yeah");
// }
//
// #[test]
// fn insert_yeah() {
//     check!(
//         "",
//         "yeah",
//         Edit::One(Command::Insert(Insert {
//             index: 0,
//             text: "yeah".into()
//         })),
//         Ok(())
//     );
// }
//
// #[test]
// fn delete_yeah() {
//     check!(
//         "yeah",
//         "",
//         Edit::One(Command::Delete(Delete { range: 0..4 })),
//         Ok(())
//     );
// }
// #[test]
// fn replace_yeah() {
//     check!(
//         "yeah",
//         "Hello, World!",
//         Edit::One(Command::Replace(Replace {
//             range: 0..4,
//             new_text: "Hello, World!",
//         })),
//         Ok(())
//     );
// }
