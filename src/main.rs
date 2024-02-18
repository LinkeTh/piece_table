use std::fmt::{Display, Formatter};

#[derive(Debug, Copy, PartialOrd, PartialEq, Clone, Ord, Eq)]
enum Buffer {
    Original,
    Add,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Piece {
    buffer: Buffer,
    length: usize,
    offset: usize,
}

impl Piece {
    fn new(buffer: Buffer, length: usize, offset: usize) -> Self {
        Piece { buffer, length, offset }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct PieceTable {
    original: String,
    add: String,
    pieces: Vec<Piece>,
}

impl PieceTable {
    fn new(original: String) -> Self {
        let original_piece = Piece::new(Buffer::Original, original.len(), 0);
        let pieces = vec![original_piece];
        let add = String::new();

        PieceTable { original, add, pieces }
    }

    fn char_at(&self, offset: usize) -> Option<char> {
        if let Some((piece, _index, running_total)) = self.piece_at(offset) {
            let content = match piece.buffer {
                Buffer::Original => self.original.chars().nth(piece.offset + offset - running_total),
                Buffer::Add => self.add.chars().nth(piece.offset + offset - running_total),
            };
            return content;
        }
        None
    }

    fn piece_at(&self, offset: usize) -> Option<(Piece, usize, usize)> {
        let mut running_total = 0;

        for (index, piece) in self.pieces.iter().enumerate() {
            let offset_start = running_total;
            let offset_end = piece.length + running_total;
            let range = offset_start..offset_end;

            if range.contains(&offset) {
                return Some((piece.clone(), index, running_total));
            }
            running_total += piece.length;
        }
        None
    }

    fn insert(&mut self, offset: usize, content: &str) {
        let total_length = self.length();

        if content.is_empty() {
            return;
        }

        if offset == 0 {
            let add_piece = Piece::new(Buffer::Add, content.len(), self.add.len());
            self.pieces.insert(0, add_piece);
            self.add.push_str(content);
        } else if offset >= total_length {
            let add_piece = Piece::new(Buffer::Add, content.len(), self.add.len());
            self.pieces.push(add_piece);
            self.add.push_str(content);
        } else {
            if let Some((piece_to_split, index, total)) = self.piece_at(offset) {
                let mut index = index;

                let remaining_length = offset.checked_sub(total).unwrap();
                let remaining_offset = piece_to_split.offset;
                let remaining = Piece::new(piece_to_split.buffer, remaining_length, remaining_offset);

                let new_length = piece_to_split.length.checked_sub(remaining_length).unwrap();
                let new_offset = remaining_length + remaining_offset;
                let new = Piece::new(piece_to_split.buffer, new_length, new_offset);

                let re_add_org = remaining_length > 0;

                let new_add = Piece::new(Buffer::Add, content.len(), self.add.len());
                self.pieces.insert(index + 1, new_add);
                self.add.push_str(content);

                if re_add_org {
                    self.pieces[index] = remaining;
                } else {
                    self.pieces.remove(index);
                    index -= 1;
                }

                self.pieces.insert(index + 2, new);
            }
        }
    }

    fn delete(&mut self, offset: usize, length: usize) {
        let total_length = self.length();

        if total_length < length && offset == 0 {
            println!("clear all");
            self.pieces.clear();
        }

        let delete_range = offset..(offset + length);
        println!("delete -> {:?}", delete_range);

        let mut running_total = 0;
        let mut split = false;

        let _ = &self.pieces.retain_mut(|piece| {
            let piece_start = running_total;
            let piece_end = piece.length + running_total;

            running_total += piece.length;

            if delete_range.contains(&piece_start) && delete_range.contains(&piece_end) {
                println!("delete -> {:?} piece_start {} piece_end {} ", piece, piece_start, piece_end);
                return false;
            } else if delete_range.contains(&piece_start) {
                println!("change start -> {:?} piece_start {} piece_end {} ", piece, piece_start, piece_end);
                let diff = delete_range.end - piece_start;
                if piece.length - diff == 0 {
                    return false;
                }
                piece.length = piece.length - diff;
                piece.offset = piece.offset + diff;
                return true;
            } else if delete_range.contains(&piece_end) {
                println!("change end -> {:?} piece_start {} piece_end {} ", piece, piece_start, piece_end);
                piece.length = delete_range.start;
                return true;
            } else if (piece_start..piece_end).contains(&delete_range.start) && (piece_start..piece_end).contains(&delete_range.end) {
                println!("split -> {:?} piece_start {} piece_end {}", piece, piece_start, piece_end);
                split = true;
                return true;
            }
            println!("keep -> {:?} piece_start {} piece_end {}", piece, piece_start, piece_end);
            true
        });

        if split {
            if let Some((piece_to_split, index, total)) = self.piece_at(offset) {
                if piece_to_split.length == length {
                    println!("remove -> {:?}", piece_to_split);
                    self.pieces.remove(index);
                } else if piece_to_split.length > length {
                    let new_length = piece_to_split.length.checked_sub(offset + length).unwrap();
                    let new_offset = offset + length + piece_to_split.offset;
                    let new = Piece::new(piece_to_split.buffer, new_length, new_offset);
                    println!("new -> {:?}", new);

                    let remaining_length = offset.checked_sub(total).unwrap();
                    let remaining_offset = piece_to_split.offset;
                    let remaining = Piece::new(piece_to_split.buffer, remaining_length, remaining_offset);
                    println!("remaining -> {:?}", remaining);

                    self.pieces[index] = remaining;
                    self.pieces.insert(index + 1, new);
                }
            }
        }
    }

    fn length(&self) -> usize {
        let mut length = 0;
        for piece in &self.pieces {
            length += piece.length;
        }
        length
    }

    fn display(&self) -> String {
        let mut result = String::new();
        for piece in &self.pieces {
            println!("display {:?}", piece);

            let content = match piece.buffer {
                Buffer::Original => &self.original[piece.offset..piece.offset + piece.length],
                Buffer::Add => &self.add[piece.offset..piece.offset + piece.length],
            };
            result.push_str(content);
        }
        result
    }
}

impl Display for PieceTable {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display())
    }
}

fn main() {
    let mut piece_table = PieceTable::new("ipsum sit amet".to_string());
    // let mut piece_table = PieceTable::new("For instance, to get the value of Index(15), the 3rd entry of piece table is retrieved. This is because the 3rd \
    // entry describes the characters from index 11 to 16 (the first entry describes characters in index 0 to 5, the next one is 6 to 10). The piece table entry instructs \
    // the program to look for the characters in the \"add file\" buffer, starting at index 17 in that buffer. The relative index in that entry is 15-11 = 4, which is added \
    // to the start position of the entry in the buffer to obtain index of the letter: 4+17 = 21. The value of Index(15) is the 21st character of the \"add file\" buffer, \
    // which is the character \"o\".".to_string());

    // piece_table.delete(0, 5);
    piece_table.insert(0, "Lorem ");
    piece_table.insert(6, "deletedtext");
    piece_table.delete(6, "deletedtext".len());
    piece_table.insert(11, " dolor");

    // piece_table.insert(0, "Lorem ");
    // piece_table.insert(11, " dolor");

    println!("1.test -> \n{}", piece_table);

    // piece_table.insert(30, " am ende");
    //
    // println!("2.test -> {}", piece_table);
    //
    // piece_table.insert(25, "0xxxxx0");
    // piece_table.insert(20, "1xxxxx1");
    // piece_table.insert(50, "2xxxxx2");
    // piece_table.insert(10, "3xxxxx3");
    // println!("3.test -> {}", piece_table);
    //
    // piece_table.insert(11, "a");
    // piece_table.insert(12, "b");
    // piece_table.insert(13, "c");
    // piece_table.insert(14, "d");
    // piece_table.insert(15, "a");
    // piece_table.insert(15, "b");
    // piece_table.insert(15, "c");
    // piece_table.insert(15, "d");
    // println!("4.test -> {}", piece_table);
    //
    // piece_table.delete(5, 38);
    //
    // println!("5.test -> {}", piece_table);
    //
    // piece_table.delete(0, 1);
    // println!("6.test -> {}", piece_table);
    //
    // println!("{:?}", piece_table.piece_at(15));
    // println!("{:?}", piece_table.char_at(15));
}

// }
// let mut running_total = 0;
//
// let _ = &self.pieces.retain_mut(|piece| {
//     let piece_start = running_total;
//     let piece_end = piece.length + running_total;
//
//     running_total += piece.length;
//
//     if delete_range.contains(&piece_start) && delete_range.contains(&piece_end) {
//         println!("delete -> {:?} piece_start {} piece_end {} ", piece, piece_start, piece_end);
//         return false;
//     } else if delete_range.contains(&piece_start) {
//         println!("change start -> {:?} piece_start {} piece_end {} ", piece, piece_start, piece_end);
//         let diff = delete_range.end - piece_start;
//         piece.length = piece.length - diff;
//         piece.offset = piece.offset + diff;
//         return true;
//     } else if delete_range.contains(&piece_end) {
//         println!("change end -> {:?} piece_start {} piece_end {} ", piece, piece_start, piece_end);
//         piece.length = delete_range.start;
//         return true;
//     }
//     println!("keep -> {:?} piece_start {} piece_end {}", piece, piece_start, piece_end);
//     true
// });

// for (index, piece) in self.pieces.iter().enumerate() {
//     let offset_start = running_total;
//     let offset_end = piece.length + running_total;
//     let range = offset_start..offset_end;
//     if range.contains(&offset) {
//         println!("remove -> {:?}", index);
//         self.pieces.remove(index);
//     }
//     running_total += piece.length;
// }

// println!("delete -> {:?} {:?} {:?}", self.add, offset, length);
// if let Some((piece_to_remove, index, total)) = self.piece_at(offset) {
//     if piece_to_remove.length == length {
//         println!("remove -> {:?}", piece_to_remove);
//         self.pieces.remove(index);
//         self.delete(offset, length);
//     } else if piece_to_remove.length > length {
//         let new_length = piece_to_remove.length.checked_sub(length).unwrap();
//         let new_offset = length + piece_to_remove.offset;
//         let new = Piece::new(piece_to_remove.buffer, new_length, new_offset);
//         println!("new -> {:?} {} {}", new, length, piece_to_remove.length);
//         self.pieces.remove(index);
//         self.pieces[index] = new;
//     } else if piece_to_remove.length < length {
//         println!("recursive -> {:?} {}", piece_to_remove, length);
//         self.pieces.remove(index);
//         self.delete(offset + piece_to_remove.length, length - piece_to_remove.length);
//     } else if total_length < length {
//         println!("clear -> {:?} {}", piece_to_remove, total);
//         self.pieces.clear();
//     } else {
//         println!("args -> {:?}", piece_to_remove);
//         //     self.pieces.remove(index);
//         //     self.delete(offset, piece_to_remove.length);
//     }
// } else {
//     println!("delete not found -> {:?}", offset);
//     if length == 0 {
//         return;
//     }
//     self.delete(offset, length - 1);
// }
