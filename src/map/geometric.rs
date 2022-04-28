use bevy::prelude::*;

use super::WithinBoxIterator;


// Components
#[derive(Component, Default, Deref, DerefMut, Clone)]
pub struct Entrances (pub Vec<Entity>);

#[derive(Component, Default, Deref, DerefMut, Clone)]
pub struct Exits (pub Vec<Entity>);

#[derive(Component, Default, Debug, Clone)]
pub struct Rect3Room {
    // We should add actors to this list when they're spawned in a room.
    // We should check to see if this list is empty before adding actors during world generation.
    // It would be bad to spawn two actors in the same location.
    pub spawned_actors: Vec<Entity>, 
    pub rect: Rect3,
    pub ceiling: Tile,
    pub walls: Tile,
    pub floor: Tile,
}
impl IntoIterator for Rect3Room {
    type Item = IVec3;

    type IntoIter = WithinBoxIterator;

    fn into_iter(self) -> Self::IntoIter {
        WithinBoxIterator::new(self.rect.min(), self.rect.max())
    }
}
impl IntoIterator for &Rect3Room {
    type Item = IVec3;

    type IntoIter = WithinBoxIterator;

    fn into_iter(self) -> Self::IntoIter {
        WithinBoxIterator::new(self.rect.min(), self.rect.max())
    }
}

#[derive(Component, Default, Deref, DerefMut, Clone)]
pub struct Path (pub Vec<IVec3>);

// Data


#[derive(Default, Debug, Clone, PartialEq)]
pub struct Tile {
    pub mesh: Handle<Mesh>,
    pub material: Handle<StandardMaterial>,
}

#[derive(Default, Copy, Clone, Debug, PartialEq)]
pub struct Rect3 {
    pub pos1: IVec3,
    pub pos2: IVec3,
}
impl Rect3 {
    pub fn new(pos: IVec3, width: i32, height: i32, length: i32) -> Rect3 {
        Rect3 {pos1: pos, pos2: IVec3::new(pos.x + width - 1, pos.y + height - 1, pos.z + length - 1)}
    }

    // Returns true if this overlaps with other
    pub fn intersect(&self, other: &Rect3) -> bool {
        let min_self = self.min();
        let max_self = self.min();

        let min_other = other.min();
        let max_other = other.max();

        min_self.x <= max_other.x && max_self.x >= min_other.x &&
        min_self.y <= max_other.y && max_self.y >= min_other.y &&
        min_self.z <= max_other.z && max_self.z >= min_other.z
    }

    pub fn center(&self) -> Vec3 { 
        Vec3::new((self.pos1.x + self.pos2.x) as f32 / 2.0, (self.pos1.y + self.pos2.y) as f32 /2.0, (self.pos1.z + self.pos2.z) as f32 / 2.0)
    }
    
    pub fn min(&self) -> IVec3 {
        IVec3::new(self.pos1.x.min(self.pos2.x), self.pos1.y.min(self.pos2.y), self.pos1.z.min(self.pos2.z))
    }

    pub fn max(&self) -> IVec3 {
        IVec3::new(self.pos1.x.max(self.pos2.x), self.pos1.y.max(self.pos2.y), self.pos1.z.max(self.pos2.z))
    }
}
impl IntoIterator for Rect3 {
    type Item = IVec3;

    type IntoIter = WithinBoxIterator;

    fn into_iter(self) -> Self::IntoIter {
        WithinBoxIterator::new(self.min(), self.max())
    }
}