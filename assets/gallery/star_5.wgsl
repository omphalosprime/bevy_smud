#import bevy_smud::shapes

fn sdf(p: vec2<f32>) -> f32 {
    return sd_star_5(p, 10., 2.);
}