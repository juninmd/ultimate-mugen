pub(super) const PER_PAGE: usize = 18;

#[derive(Clone, Copy, PartialEq, Debug)]
pub(super) enum Dir {
    Left,
    Right,
    Up,
    Down,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub(super) enum Move {
    To(usize),
    Button,
}

pub(super) fn page_count(total: usize) -> usize {
    total.div_ceil(PER_PAGE).max(1)
}

pub(super) fn page_len(total: usize, page: usize) -> usize {
    total.saturating_sub(page * PER_PAGE).min(PER_PAGE)
}

pub(super) fn go(cur: usize, total: usize, dir: Dir, cols_of: &dyn Fn(usize) -> usize) -> Move {
    if total == 0 {
        return Move::Button;
    }
    let start = cur / PER_PAGE * PER_PAGE;
    let count = page_len(total, cur / PER_PAGE);
    let cols = cols_of(count).max(1);
    let col = (cur - start) % cols;
    match dir {
        Dir::Left => Move::To(cur.saturating_sub(1)),
        Dir::Right => Move::To((cur + 1).min(total - 1)),
        Dir::Down => {
            if cur + cols < start + count {
                Move::To(cur + cols)
            } else if start + count < total {
                Move::To((start + count + col).min(total - 1))
            } else {
                Move::To(start + count - 1)
            }
        }
        Dir::Up => {
            if cur - start >= cols {
                Move::To(cur - cols)
            } else if start == 0 {
                Move::Button
            } else {
                let prev = start - PER_PAGE;
                let prev_cols = cols_of(PER_PAGE).max(1);
                let last_row = prev + (PER_PAGE - 1) / prev_cols * prev_cols;
                Move::To((last_row + col).min(prev + PER_PAGE - 1))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn six(_: usize) -> usize {
        6
    }

    #[test]
    fn pages_cover_every_character() {
        assert_eq!(page_count(0), 1);
        assert_eq!(page_count(18), 1);
        assert_eq!(page_count(19), 2);
        assert_eq!(page_len(40, 2), 4);
    }

    #[test]
    fn down_crosses_to_the_next_page_keeping_the_column() {
        assert_eq!(go(14, 40, Dir::Down, &six), Move::To(20));
        assert_eq!(go(2, 40, Dir::Down, &six), Move::To(8));
    }

    #[test]
    fn up_returns_to_the_previous_page_last_row() {
        assert_eq!(go(20, 40, Dir::Up, &six), Move::To(14));
        assert_eq!(go(3, 40, Dir::Up, &six), Move::Button);
    }

    #[test]
    fn edges_never_leave_the_roster() {
        assert_eq!(go(0, 5, Dir::Left, &six), Move::To(0));
        assert_eq!(go(4, 5, Dir::Right, &six), Move::To(4));
        assert_eq!(go(4, 5, Dir::Down, &six), Move::To(4));
        assert_eq!(go(0, 0, Dir::Down, &six), Move::Button);
    }

    #[test]
    fn down_on_a_short_last_row_lands_on_the_last_card() {
        assert_eq!(go(1, 4, Dir::Down, &|_| 3), Move::To(3));
    }
}
