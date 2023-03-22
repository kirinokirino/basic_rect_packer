use glam::UVec2;
use glam_rect::URect;

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub(crate) enum TexturePackerError {
    NotEnoughSpace,
}

#[derive(Debug, Clone)]
pub(crate) struct TexturePacker {
    pub areas: Vec<URect>,
}

impl TexturePacker {
    pub(crate) fn new(width: u32, height: u32) -> Self {
        let top_left = UVec2::new(0, 0);
        let bottom_right = UVec2::new(width, height);
        TexturePacker {
            areas: vec![URect::new(top_left, bottom_right)],
        }
    }

    pub(crate) fn try_allocate(&mut self, size: UVec2) -> Result<URect, TexturePackerError> {
        if size.x == 0 || size.y == 0 {
            return Ok(URect::new(UVec2::ZERO, size));
        }

        let size = size + UVec2::new(2, 2);

        // Add a one-pixel border around each texture
        let width = size.x;
        let height = size.y;

        let mut best_area: Option<&mut URect> = None;

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

        let best_area = best_area.ok_or(TexturePackerError::NotEnoughSpace)?;
        let URect {
            top_left,
            bottom_right,
        } = best_area.clone();

        let space_underneath =
            URect::new(UVec2::new(top_left.x, (top_left + size).y), bottom_right);

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
        let mut packer = TexturePacker::new(64, 64);

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
            Err(TexturePackerError::NotEnoughSpace),
            packer.try_allocate(UVec2::new(30, 30))
        );
    }

    #[test]
    fn pack_test_nonfill_four_squares() {
        let mut packer = TexturePacker::new(64, 64);

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
            Err(TexturePackerError::NotEnoughSpace),
            packer.try_allocate(UVec2::new(30, 30))
        );
    }

    #[test]
    fn pack_test_uneven_squares() {
        let mut packer = TexturePacker::new(64, 64);

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
