use std::fmt::{Display, Formatter};

#[derive(Debug, Copy, PartialOrd, PartialEq, Clone, Ord, Eq)]
enum Buffer {
    Original,
    Add,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Piece {
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
pub struct PieceTable {
    original: String,
    add: String,
    pieces: Vec<Piece>,
}

impl PieceTable {
    pub fn new(original: String) -> Self {
        let original_piece = Piece::new(Buffer::Original, original.len(), 0);
        let pieces = vec![original_piece];
        let add = String::new();

        PieceTable { original, add, pieces }
    }

    pub fn char_at(&self, offset: usize) -> Option<char> {
        if let Some((piece, _index, running_total)) = self.piece_at(offset) {
            let content = match piece.buffer {
                Buffer::Original => self.original.chars().nth(piece.offset + offset - running_total),
                Buffer::Add => self.add.chars().nth(piece.offset + offset - running_total),
            };
            return content;
        }
        None
    }

    pub fn piece_at(&self, offset: usize) -> Option<(Piece, usize, usize)> {
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

    pub fn insert(&mut self, offset: usize, content: &str) {
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

    pub fn delete(&mut self, offset: usize, length: usize) {
        let total_length = self.length();

        if total_length < length && offset == 0 {
            self.pieces.clear();
            return;
        }

        let delete_range = offset..(offset + length);

        let mut running_total = 0;
        let mut split = false;

        let _ = &self.pieces.retain_mut(|piece| {
            let piece_start = running_total;
            let piece_end = piece.length + running_total;

            running_total += piece.length;

            if delete_range.contains(&piece_start) && delete_range.contains(&piece_end) {
                return false;
            } else if delete_range.contains(&piece_start) {
                let diff = delete_range.end - piece_start;
                if piece.length - diff == 0 {
                    return false;
                }
                piece.length = piece.length - diff;
                piece.offset = piece.offset + diff;
                return true;
            } else if delete_range.contains(&piece_end) {
                piece.length = delete_range.start;
                return true;
            } else if delete_range.start > piece_start && delete_range.end < piece_end {
                split = true;
                return true;
            }
            true
        });

        if split {
            if let Some((piece_to_split, index, total)) = self.piece_at(offset) {
                if piece_to_split.length == length {
                    self.pieces.remove(index);
                } else if piece_to_split.length > length {
                    let new_length = piece_to_split.length.checked_sub(offset + length).unwrap();
                    let new_offset = offset + length + piece_to_split.offset;
                    let new = Piece::new(piece_to_split.buffer, new_length, new_offset);

                    let remaining_length = offset.checked_sub(total).unwrap();
                    let remaining_offset = piece_to_split.offset;
                    let remaining = Piece::new(piece_to_split.buffer, remaining_length, remaining_offset);

                    self.pieces[index] = remaining;
                    self.pieces.insert(index + 1, new);
                }
            }
        }
    }

    pub fn length(&self) -> usize {
        let mut length = 0;
        for piece in &self.pieces {
            length += piece.length;
        }
        length
    }

    pub fn text(&self) -> String {
        let mut result = String::new();
        for piece in &self.pieces {
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
        write!(f, "{}", self.text())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_it() {
        let mut piece_table = PieceTable::new("ipsum sit amet".to_string());

        piece_table.insert(0, "Lorem ");
        piece_table.insert(6, "deletedtext");
        piece_table.delete(6, "deletedtext".len());
        piece_table.insert(11, " dolor");

        assert_eq!("Lorem ipsum dolor sit amet", piece_table.text());
        assert_eq!('o', piece_table.char_at(15).unwrap());
    }
}
