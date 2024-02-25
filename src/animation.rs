use bevy::prelude::*;

#[derive(Component, Default)]
pub(crate) struct AnimationTimer {
    pub(crate) timer: Timer,
}

#[derive(Component, Clone, Default)]
pub(crate) struct AnimationIndices {
    pub(crate) first: usize,
    pub(crate) last: usize,
}

pub(crate) fn animate(
    time: Res<Time>,
    mut animated_sprites: Query<(&mut AnimationTimer, &AnimationIndices, &mut TextureAtlas)>,
) {
    for (mut timer, indices, mut sprite) in &mut animated_sprites {
        timer.timer.tick(time.delta());
        if timer.timer.finished() {
            let mut new_index = sprite.index + 1;
            if new_index > indices.last {
                new_index = indices.first;
            }
            sprite.index = new_index;
        }
    }
}
