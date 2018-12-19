use ggez::{Context, graphics::{self, DrawParam, Point2}};

pub fn with_translate<F, R>(
    ctx: &mut Context, translate: Point2, f: F) -> R
    where F: FnOnce(&mut Context) -> R
{
    graphics::push_transform(ctx, Some(graphics::get_transform(ctx) * DrawParam {
        dest: translate, ..Default::default()
    }.into_matrix()));
    graphics::apply_transformations(ctx);

    let result = f(ctx);

    graphics::pop_transform(ctx);
    graphics::apply_transformations(ctx);

    result
}
