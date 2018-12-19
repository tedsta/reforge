use ggez::{
    Context, GameResult,
    event::{Event, Events},
    graphics,
};

pub trait GameState {
    type Context;
    type Action;

    fn event(&mut self, gtx: &mut Self::Context, e: &Event) -> Option<Self::Action>;
    fn update(&mut self, gtx: &mut Self::Context) -> Option<Self::Action> { None } // optional
    fn draw(&mut self, gtx: &mut Self::Context, ctx: &mut Context) -> GameResult<()>;
}

pub fn run<S: GameState>(
    gtx: &mut S::Context, ctx: &mut Context, state: &mut S)
    -> GameResult<Option<S::Action>>
{
    use Event::*;

    let mut events = Events::new(ctx)?;
    loop {
        ctx.timer_context.tick();

        if let Some(result) = state.update(gtx) {
            return Ok(Some(result));
        }

        for e in events.poll() {
            match e {
                Quit { .. } => { break; },
                /*Window {
                    win_event: event::WindowEvent::FocusGained,
                    ..
                } => { },
                Window {
                    win_event: event::WindowEvent::FocusLost,
                    ..
                } => { },
                Window {
                    win_event: event::WindowEvent::Resized(w, h),
                    ..
                } => { },*/
                _ => {
                    if let Some(result) = state.event(gtx, &e) {
                        return Ok(Some(result));
                    }
                }
            }
        }

        draw_state(gtx, ctx, state)?;
    }

    Ok(None)
}

fn draw_state<S: GameState>(
    gtx: &mut S::Context, ctx: &mut Context, state: &mut S) -> GameResult<()>
{
    graphics::set_color(ctx, [0.0, 0.0, 0.0, 0.0].into())?;
    graphics::clear(ctx);
    graphics::set_color(ctx, [1.0, 1.0, 1.0, 1.0].into())?;

    state.draw(gtx, ctx)?;

    graphics::present(ctx);
    ggez::timer::yield_now();

    Ok(())
}
