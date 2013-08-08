extern mod kiss3d;
extern mod nalgebra;

use std::num::Zero;
use nalgebra::vec::Vec3;
use kiss3d::window;
use kiss3d::camera;

fn main()
{
    do window::Window::spawn("Kiss3d: camera") |w|
    {
        do w.set_loop_callback
        {
            w.draw_line(&Zero::zero(), &Vec3::x(), &Vec3::x());
            w.draw_line(&Zero::zero(), &Vec3::y(), &Vec3::y());
            w.draw_line(&Zero::zero(), &Vec3::z(), &Vec3::z());

            do w.camera().change_mode |mode|
            {
                match *mode
                {
                    camera::ArcBall(ref mut ab) => ab.yaw = ab.yaw + 0.05,
                    _                           => { }
                }
            }
        }

        w.set_light(window::StickToCamera);
    }
}
