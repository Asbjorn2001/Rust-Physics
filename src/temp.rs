/*
AABB:

let dx = self.top_left.curr.x - r.top_left.curr.x;
let dy = self.top_left.curr.y - r.top_left.curr.y;
let avg_width = (self.size.x + r.size.x) / 2.0;
let avg_height = (self.size.y + r.size.y) / 2.0;
if dx.abs() < avg_height && dy.abs() < avg_height {
    let px = avg_width - dx.abs();
    let py = avg_height - dy.abs();
    if px < py {
        if dx > 0.0 {
            self.velocity.next.x = -self.velocity.curr.x;
            self.top_left.next.x = r.top_left.curr.x + r.size.x;
        } else {
            self.velocity.next.x = -self.velocity.curr.x;
            self.top_left.next.x = r.top_left.curr.x - self.size.x;
        }
    } else {
        if dy > 0.0 {
            self.velocity.next.y = -self.velocity.curr.y;
            self.top_left.next.y = r.top_left.curr.y + r.size.y;
        } else {
            self.velocity.next.y = -self.velocity.curr.y;
            self.top_left.next.y = r.top_left.curr.y - self.size.y;
        }
    }
}
*/


/*
let rv = r.velocity.curr - self.velocity.curr;
let vel_along_normal = rv.dot(normal);

if vel_along_normal > 0.0 {
    return;
}

let m1 = self.get_area(); let m2 = r.get_area();
let impulse = -2.0 * vel_along_normal / (1.0 / m1 + 1.0 / m2);

let impulse_vector = normal * impulse;
self.velocity.next = self.velocity.curr + (impulse_vector / m1);
*/


/* 
                let collision = self.swept_aabb(r);   
                match collision {
                    Some((normal, entry_time)) => {                        
                        if entry_time > dt {
                            return;
                        }

                        self.top_left.next = self.top_left.curr + self.velocity.curr * entry_time / 2.0;
                        
                        if normal.x != 0.0 {
                            self.velocity.next.x = -self.velocity.curr.x;
                        } else {
                            self.velocity.next.y = -self.velocity.curr.y;
                        }
                    }
                    None => {},
                }
                */


/*
impl Rectangle {
    // Returns an option "collision normal vector", or none if no collision occured
    fn swept_aabb(&self, other: Rectangle) -> Option<(Vector2<f64>, f64)> {
        let dynamic_pos = self.center.curr;
        let static_pos = other.center.curr;

        let rv = self.velocity.curr - other.velocity.curr;
        let x_entry; let x_exit; let y_entry; let y_exit; 

        if rv.x > 0.0 {
            x_entry = static_pos.x - (dynamic_pos.x + other.dims.x);        
            x_exit = (static_pos.x + self.dims.x) - dynamic_pos.x;
        } else {
            x_entry = (static_pos.x + self.dims.x) - dynamic_pos.x;
            x_exit = static_pos.x - (dynamic_pos.x + other.dims.x);  
        }

        if rv.y > 0.0 {
            y_entry = static_pos.y - (dynamic_pos.y + other.dims.y);
            y_exit = (static_pos.y + self.dims.y) - dynamic_pos.y;
        } else {
            y_entry = (static_pos.y + self.dims.y) - dynamic_pos.y;
            y_exit = static_pos.y - (dynamic_pos.y + other.dims.y);
        }

        let x_entry_time = if rv.x != 0.0 { x_entry / rv.x } else { f64::NEG_INFINITY };
        let x_exit_time = if rv.x != 0.0 { x_exit / rv.x } else { f64::INFINITY };

        let y_entry_time = if rv.y != 0.0 { y_entry / rv.y } else { f64::NEG_INFINITY };
        let y_exit_time = if rv.y != 0.0 { y_exit / rv.y } else { f64::INFINITY };

        let entry_time = x_entry_time.max(y_entry_time);
        let exit_time = x_exit_time.min(y_exit_time);

        if entry_time > exit_time || (x_entry_time < 0.0 && y_entry_time < 0.0) || entry_time > 1.0 {
            return None;
        }

        if x_entry_time > y_entry_time {
            if x_entry < 0.0 {
                return Some((Vector2::new(-1.0, 0.0), x_entry_time));
            } else {
                return Some((Vector2::new(1.0, 0.0), x_entry_time));
            }
        } else {
            if y_entry < 0.0 {
                return Some((Vector2::new(0.0, -1.0), y_entry_time));
            } else {
                return Some((Vector2::new(0.0, 1.0), y_entry_time));
            }
        }        
    }
}
*/