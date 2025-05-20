use super::draw_target::DrawTarget;

pub fn draw_hline<Target: DrawTarget>(target: &mut Target, x_start: isize, x_end: isize, y: isize, color: bool) {
    if x_start <= x_end {
        for x in x_start..=x_end {
            target.set_pixel((x, y), color);
        }
    } else {
        for x in x_end..=x_start {
            target.set_pixel((x, y), color);
        }
    }
}

pub fn draw_vline<Target: DrawTarget>(target: &mut Target, x: isize, y_start: isize, y_end: isize, color: bool) {
    if y_start <= y_end {
        for y in y_start..=y_end {
            target.set_pixel((x, y), color);
        }
    } else {
        for y in y_end..=y_start {
            target.set_pixel((x, y), color);
        }
    }
}

pub fn draw_rect<Target: DrawTarget>(target: &mut Target, start: (isize, isize), end: (isize, isize), color: bool) {
    draw_hline(target, start.0, end.0, start.1, color);
    draw_hline(target, start.0, end.0, end.1, color);
    draw_vline(target, start.0, start.1, end.1, color);
    draw_vline(target, end.0, start.1, end.1, color);
}

pub fn draw_filled_rect<Target: DrawTarget>(target: &mut Target, start: (isize, isize), end: (isize, isize), color: bool) {
    let x_min = start.0.min(end.0);
    let x_max = start.0.max(end.0);
    let y_min = start.1.min(end.1);
    let y_max = start.1.max(end.1);
    for y in y_min..=y_max {
        for x in x_min..=x_max {
            target.set_pixel((x, y), color);
        }
    }
}

pub fn draw_line<Target: DrawTarget>(target: &mut Target, start: (isize, isize), end: (isize, isize), color: bool) {
    let dx = end.0 - start.0;
    let dy = end.1 - start.1;
    match (dx.abs(), dy.abs()) {
        (0, 0) => target.set_pixel(start, color),
        (0, _) => draw_vline(target, start.0, start.1, end.1, color),
        (_, 0) => draw_hline(target, start.0, end.0, start.1, color),
        (adx, ady) => {
            if adx >= ady {
                if start.0 < end.0 {
                    for x in start.0..=end.0 {
                        let tdx = x - start.0;
                        let tdy = (dy * tdx) / dx;
                        let y = start.1 + tdy;
                        target.set_pixel((x, y), color);
                    }
                } else {
                    for x in end.0..=start.0 {
                        let tdx = x - start.0;
                        let tdy = (dy * tdx) / dx;
                        let y = start.1 + tdy;
                        target.set_pixel((x, y), color);
                    }
                }
            } else {
                if start.1 < end.1 {
                    for y in start.1..=end.1 {
                        let tdy = y - start.1;
                        let tdx = (dx * tdy) / dy;
                        let x = start.0 + tdx;
                        target.set_pixel((x, y), color);
                    }
                } else {
                    for y in end.1..=start.1 {
                        let tdy = y - start.1;
                        let tdx = (dx * tdy) / dy;
                        let x = start.0 + tdx;
                        target.set_pixel((x, y), color);
                    }
                }
            }
        }
    }
}
