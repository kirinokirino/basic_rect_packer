use glam::UVec2;
use glam_rect::URect;

static DEFAULT_ADMISSIBLE_WASTE: u8 = 8;

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum PackerError {
    NotEnoughSpace,
}

#[derive(Debug, Clone)]
pub struct Packer {
    pub areas: Vec<URect>,
    pub admissable_waste: u32,
}

impl Packer {
    pub fn new(width: u32, height: u32) -> Self {
        let top_left = UVec2::new(0, 0);
        let bottom_right = UVec2::new(width, height);
        Packer {
            areas: vec![URect::new(top_left, bottom_right)],
            admissable_waste: DEFAULT_ADMISSIBLE_WASTE.into(),
        }
    }

    pub fn with_admissable_waste(mut self, waste: u32) -> Self {
        self.admissable_waste = waste;
        self
    }

    pub fn pack(&mut self, mut sizes: Vec<UVec2>) -> Vec<Result<URect, PackerError>> {
        if sizes.is_empty() {
            return Vec::new();
        }
        sizes.sort_unstable_by(|a, b| a.y.cmp(&b.y));
        let padding = 2;
        let smallest = sizes.first().unwrap();
        // if we can't place even the smallest item there, space can be wasted.
        self.admissable_waste = smallest.y - 1 + padding;
        sizes
            .into_iter()
            .rev()
            .map(|size| self.try_allocate(size))
            .collect()
    }

    pub fn try_allocate(&mut self, size: UVec2) -> Result<URect, PackerError> {
        if size.x == 0 || size.y == 0 {
            return Ok(URect::new(UVec2::ZERO, size));
        }

        let mut best_area: Option<&mut URect> = None;

        // Add a one-pixel border around each texture
        let size = size + UVec2::new(2, 2);
        let width = size.x;
        let height = size.y;
        for area in &mut self.areas {
            let area_width = area.width();
            let area_height = area.height();

            if width > area.width() || height > area.height() {
                continue;
            }

            let update_best = if let Some(current_best) = &best_area {
                current_best.width() >= area_width && current_best.height() >= area_height
            } else {
                true
            };

            if update_best {
                best_area = Some(area);
            }
        }

        let best_area = best_area.ok_or(PackerError::NotEnoughSpace)?;
        let URect {
            top_left,
            bottom_right,
        } = best_area.clone();

        let new_height = (top_left + size).y;
        let new_height = if (bottom_right.y - new_height) > self.admissable_waste {
            new_height
        } else {
            bottom_right.y
        };
        let space_underneath = URect::new(UVec2::new(top_left.x, new_height), bottom_right);

        let space_right = URect::new(
            UVec2::new((top_left + size).x, top_left.y),
            space_underneath.top_right(),
        );

        if space_right.is_zero_area() {
            *best_area = space_underneath
        } else {
            *best_area = space_right;

            if !space_underneath.is_zero_area() {
                self.areas.push(space_underneath);
            }
        }

        Ok(URect::new(
            top_left + UVec2::new(1, 1),
            (top_left + size) - UVec2::new(1, 1),
        ))
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn pack_test_fill_four_squares() {
        let mut packer = Packer::new(64, 64);

        assert_eq!(
            Ok(URect::from_tuples((1, 1), (31, 31))),
            packer.try_allocate(UVec2::new(30, 30))
        );

        assert_eq!(
            Ok(URect::from_tuples((33, 1), (63, 31))),
            packer.try_allocate(UVec2::new(30, 30))
        );

        assert_eq!(
            Ok(URect::from_tuples((1, 33), (31, 63))),
            packer.try_allocate(UVec2::new(30, 30))
        );

        assert_eq!(
            Ok(URect::from_tuples((33, 33), (63, 63))),
            packer.try_allocate(UVec2::new(30, 30))
        );

        assert_eq!(
            Err(PackerError::NotEnoughSpace),
            packer.try_allocate(UVec2::new(30, 30))
        );
    }

    #[test]
    fn pack_test_nonfill_four_squares() {
        let mut packer = Packer::new(64, 64);

        assert_eq!(
            Ok(URect::from_tuples((1, 1), (29, 29))),
            packer.try_allocate(UVec2::new(28, 28))
        );

        assert_eq!(
            Ok(URect::from_tuples((31, 1), (59, 29))),
            packer.try_allocate(UVec2::new(28, 28))
        );

        assert_eq!(
            Ok(URect::from_tuples((1, 31), (29, 59))),
            packer.try_allocate(UVec2::new(28, 28))
        );

        assert_eq!(
            Ok(URect::from_tuples((31, 31), (59, 59))),
            packer.try_allocate(UVec2::new(28, 28))
        );

        assert_eq!(
            Err(PackerError::NotEnoughSpace),
            packer.try_allocate(UVec2::new(30, 30))
        );
    }

    #[test]
    fn pack_test_uneven_squares() {
        let mut packer = Packer::new(64, 64);

        assert_eq!(
            Ok(URect::from_tuples((1, 1), (15, 15))),
            packer.try_allocate(UVec2::new(14, 14))
        );

        assert_eq!(
            Ok(URect::from_tuples((1, 17), (15, 47))),
            packer.try_allocate(UVec2::new(14, 30))
        );

        assert_eq!(
            Ok(URect::from_tuples((17, 17), (47, 47))),
            packer.try_allocate(UVec2::new(30, 30))
        );

        assert_eq!(
            Ok(URect::from_tuples((17, 1), (31, 15))),
            packer.try_allocate(UVec2::new(14, 14))
        );
    }
}
