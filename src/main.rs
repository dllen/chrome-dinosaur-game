use macroquad::prelude::*;

const GRAVITY: f32 = 800.0;
const JUMP_FORCE: f32 = -400.0;
const GROUND_Y: f32 = 400.0;
const GROUND_Y2: f32 = 450.0; // 第二条地面线，距离第一条50像素
const DINO_WIDTH: f32 = 25.0;
const DINO_HEIGHT: f32 = 35.0;
const DINO_X: f32 = 120.0;
const CACTUS_WIDTH: f32 = 20.0;
const CACTUS_HEIGHT: f32 = 60.0;
const GAME_SPEED: f32 = 200.0;
const STAR_SIZE: f32 = 15.0;

#[derive(Clone)]
struct Cactus {
    x: f32,
    y: f32,
}

#[derive(Clone)]
struct Star {
    x: f32,
    y: f32,
}

#[derive(Clone)]
struct Stone {
    x: f32,
    y: f32,
    size: f32,
    color: Color,
}

#[derive(Clone)]
struct MathQuestion {
    question: String,
    answer: i32,
}

impl MathQuestion {
    fn new() -> Self {
        use ::rand::Rng;
        let mut rng = ::rand::thread_rng();

        let question_type = rng.gen_range(0..3);

        match question_type {
            0 => {
                // 两位数加法
                let a = rng.gen_range(10..99);
                let b = rng.gen_range(10..99);
                let answer = a + b;
                let question = format!("{} + {} = ?", a, b);

                Self { question, answer }
            }
            1 => {
                // 两位数减法
                let a = rng.gen_range(20..99);
                let b = rng.gen_range(10..a);
                let answer = a - b;
                let question = format!("{} - {} = ?", a, b);

                Self { question, answer }
            }
            _ => {
                // 九九乘法表
                let a = rng.gen_range(2..10);
                let b = rng.gen_range(2..10);
                let answer = a * b;
                let question = format!("{} × {} = ?", a, b);

                Self { question, answer }
            }
        }
    }
}

struct GameState {
    dino_y: f32,
    dino_velocity: f32,
    is_jumping: bool,
    cacti: Vec<Cactus>,
    stars: Vec<Star>,
    stones: Vec<Stone>,
    score: u32,
    game_over: bool,
    spawn_timer: f32,
    star_spawn_timer: f32,
    stone_spawn_timer: f32,
    font: Option<Font>,
    // 复活系统
    showing_math_question: bool,
    math_question: Option<MathQuestion>,
    input_buffer: String,
}

impl GameState {
    fn new() -> Self {
        Self {
            dino_y: GROUND_Y - DINO_HEIGHT,
            dino_velocity: 0.0,
            is_jumping: false,
            cacti: Vec::new(),
            stars: Vec::new(),
            stones: Vec::new(),
            score: 0,
            game_over: false,
            spawn_timer: 0.0,
            star_spawn_timer: 0.0,
            stone_spawn_timer: 0.0,
            font: None,
            showing_math_question: false,
            math_question: None,
            input_buffer: String::new(),
        }
    }

    fn reset(&mut self) {
        self.dino_y = GROUND_Y - DINO_HEIGHT;
        self.dino_velocity = 0.0;
        self.is_jumping = false;
        self.cacti.clear();
        self.stars.clear();
        self.stones.clear();
        self.score = 0;
        self.game_over = false;
        self.spawn_timer = 0.0;
        self.star_spawn_timer = 0.0;
        self.stone_spawn_timer = 0.0;
        self.showing_math_question = false;
        self.math_question = None;
        self.input_buffer.clear();
    }

    async fn load_font(&mut self) {
        // 尝试加载系统中文字体
        if let Ok(font_data) = load_file("assets/font.ttf").await {
            self.font = load_ttf_font_from_bytes(&font_data).ok();
        }

        // 如果没有找到字体文件，使用默认字体但显示英文
        if self.font.is_none() {
            println!("Warning: Chinese font not found, using English text");
        }
    }

    fn handle_math_input(&mut self) {
        // 处理数字输入
        for key_code in [
            KeyCode::Key0,
            KeyCode::Key1,
            KeyCode::Key2,
            KeyCode::Key3,
            KeyCode::Key4,
            KeyCode::Key5,
            KeyCode::Key6,
            KeyCode::Key7,
            KeyCode::Key8,
            KeyCode::Key9,
        ] {
            if is_key_pressed(key_code) {
                let digit = match key_code {
                    KeyCode::Key0 => '0',
                    KeyCode::Key1 => '1',
                    KeyCode::Key2 => '2',
                    KeyCode::Key3 => '3',
                    KeyCode::Key4 => '4',
                    KeyCode::Key5 => '5',
                    KeyCode::Key6 => '6',
                    KeyCode::Key7 => '7',
                    KeyCode::Key8 => '8',
                    KeyCode::Key9 => '9',
                    _ => continue,
                };
                if self.input_buffer.len() < 4 {
                    self.input_buffer.push(digit);
                }
            }
        }

        // 处理退格键
        if is_key_pressed(KeyCode::Backspace) && !self.input_buffer.is_empty() {
            self.input_buffer.pop();
        }

        // 处理回车键提交答案
        if is_key_pressed(KeyCode::Enter) && !self.input_buffer.is_empty() {
            if let Ok(answer) = self.input_buffer.parse::<i32>() {
                if let Some(question) = &self.math_question {
                    if answer == question.answer {
                        // 答对了，复活
                        self.revive();
                    } else {
                        // 答错了，重新生成题目让玩家再试
                        self.math_question = Some(MathQuestion::new());
                        self.input_buffer.clear();
                    }
                }
            }
        }

        // ESC 键取消复活，真正游戏结束
        if is_key_pressed(KeyCode::Escape) {
            self.showing_math_question = false;
        }
    }

    fn revive(&mut self) {
        self.game_over = false;
        self.showing_math_question = false;
        self.dino_y = GROUND_Y - DINO_HEIGHT;
        self.dino_velocity = 0.0;
        self.is_jumping = false;
        // 清除附近的仙人掌给玩家一些缓冲时间
        self.cacti.retain(|cactus| cactus.x > DINO_X + 100.0);
        self.input_buffer.clear();
    }

    fn update(&mut self, dt: f32) {
        // 处理数学题界面
        if self.showing_math_question {
            self.handle_math_input();
            return;
        }

        if self.game_over {
            return;
        }

        // 恐龙跳跃逻辑
        if is_key_pressed(KeyCode::Space) && !self.is_jumping {
            self.dino_velocity = JUMP_FORCE;
            self.is_jumping = true;
        }

        // 应用重力
        self.dino_velocity += GRAVITY * dt;
        self.dino_y += self.dino_velocity * dt;

        // 检查着地
        if self.dino_y >= GROUND_Y - DINO_HEIGHT {
            self.dino_y = GROUND_Y - DINO_HEIGHT;
            self.dino_velocity = 0.0;
            self.is_jumping = false;
        }

        // 生成仙人掌
        self.spawn_timer += dt;
        if self.spawn_timer > 2.5 {
            use ::rand::Rng;
            let mut rng = ::rand::thread_rng();

            // 40% 概率生成仙人掌
            if rng.gen_bool(0.4) {
                // 仙人掌底部位置在两条地面线中间
                let middle_y = GROUND_Y + (GROUND_Y2 - GROUND_Y) / 2.0;
                let cactus_y = middle_y - CACTUS_HEIGHT;
                
                // 30% 概率生成两个连在一起的仙人掌
                if rng.gen_bool(0.3) {
                    // 生成第一个仙人掌
                    self.cacti.push(Cactus {
                        x: screen_width(),
                        y: cactus_y,
                    });
                    // 生成第二个仙人掌，紧挨着第一个
                    self.cacti.push(Cactus {
                        x: screen_width() + CACTUS_WIDTH + 5.0, // 留5像素间隙
                        y: cactus_y,
                    });
                } else {
                    // 生成单个仙人掌
                    self.cacti.push(Cactus {
                        x: screen_width(),
                        y: cactus_y,
                    });
                }
            }
            self.spawn_timer = 0.0;
        }

        // 生成五角星奖励
        self.star_spawn_timer += dt;
        if self.star_spawn_timer > 1.5 {
            use ::rand::Rng;
            let mut rng = ::rand::thread_rng();

            // 80% 概率生成五角星
            if rng.gen_bool(0.8) {
                self.stars.push(Star {
                    x: screen_width(),
                    y: GROUND_Y - 40.0 - rng.gen_range(0.0..60.0), // 降低高度，在较低空中随机
                });
            }
            self.star_spawn_timer = 0.0;
        }

        // 生成地面小石子装饰
        self.stone_spawn_timer += dt;
        if self.stone_spawn_timer > 0.2 {
            use ::rand::Rng;
            let mut rng = ::rand::thread_rng();

            // 95% 概率生成小石子，更密集
            if rng.gen_bool(0.95) {
                // 重点在两条地面线中间生成彩色石子
                for _ in 0..rng.gen_range(4..8) {
                    let stone_y = if rng.gen_bool(0.7) {
                        // 70% 概率在两条线中间生成彩色石子
                        GROUND_Y + rng.gen_range(5.0..45.0)
                    } else {
                        // 30% 概率在地面线附近生成普通石子
                        let ground_line = if rng.gen_bool(0.5) { GROUND_Y } else { GROUND_Y2 };
                        ground_line + rng.gen_range(-8.0..12.0)
                    };
                    
                    // 生成随机颜色
                    let random_colors = [
                        Color::new(0.8, 0.2, 0.2, 1.0), // 红色
                        Color::new(0.2, 0.8, 0.2, 1.0), // 绿色
                        Color::new(0.2, 0.2, 0.8, 1.0), // 蓝色
                        Color::new(0.8, 0.8, 0.2, 1.0), // 黄色
                        Color::new(0.8, 0.2, 0.8, 1.0), // 紫色
                        Color::new(0.2, 0.8, 0.8, 1.0), // 青色
                        Color::new(0.9, 0.5, 0.2, 1.0), // 橙色
                        DARKGRAY, // 灰色
                        GRAY,     // 浅灰色
                    ];
                    
                    let color = if stone_y > GROUND_Y && stone_y < GROUND_Y2 {
                        // 两条线中间的石子使用随机颜色
                        random_colors[rng.gen_range(0..7)] // 不包括灰色
                    } else {
                        // 地面线附近的石子使用灰色
                        random_colors[rng.gen_range(7..9)]
                    };
                    
                    self.stones.push(Stone {
                        x: screen_width() + rng.gen_range(0.0..60.0),
                        y: stone_y,
                        size: rng.gen_range(1.5..4.5),
                        color,
                    });
                }
            }
            self.stone_spawn_timer = 0.0;
        }

        // 移动仙人掌
        for cactus in &mut self.cacti {
            cactus.x -= GAME_SPEED * dt;
        }

        // 移动五角星
        for star in &mut self.stars {
            star.x -= GAME_SPEED * dt;
        }

        // 移动小石子
        for stone in &mut self.stones {
            stone.x -= GAME_SPEED * dt;
        }

        // 移除屏幕外的仙人掌并增加分数
        let initial_count = self.cacti.len();
        self.cacti.retain(|cactus| cactus.x > -CACTUS_WIDTH);
        let removed_count = initial_count - self.cacti.len();
        self.score += removed_count as u32 * 10;

        // 移除屏幕外的五角星
        self.stars.retain(|star| star.x > -STAR_SIZE);

        // 移除屏幕外的小石子
        self.stones.retain(|stone| stone.x > -10.0);

        // 五角星碰撞检测（收集奖励）
        let dino_rect = Rect::new(DINO_X, self.dino_y, DINO_WIDTH, DINO_HEIGHT);
        let initial_star_count = self.stars.len();
        self.stars.retain(|star| {
            let star_rect = Rect::new(star.x, star.y, STAR_SIZE, STAR_SIZE);
            !dino_rect.overlaps(&star_rect)
        });
        let collected_stars = initial_star_count - self.stars.len();
        self.score += collected_stars as u32 * 5;

        // 仙人掌碰撞检测
        for cactus in &self.cacti {
            let cactus_rect = Rect::new(cactus.x, cactus.y, CACTUS_WIDTH, CACTUS_HEIGHT);
            if dino_rect.overlaps(&cactus_rect) {
                self.game_over = true;
                // 每次游戏结束都自动弹出数学题
                self.showing_math_question = true;
                self.math_question = Some(MathQuestion::new());
                self.input_buffer.clear();
                break;
            }
        }
    }

    fn draw(&self) {
        clear_background(WHITE);

        // 绘制地面
        draw_line(0.0, GROUND_Y, screen_width(), GROUND_Y, 2.0, BLACK);
        
        // 绘制第二条地面线
        draw_line(0.0, GROUND_Y2, screen_width(), GROUND_Y2, 2.0, DARKGRAY);

        // 绘制恐龙身体
        draw_rectangle(DINO_X, self.dino_y, DINO_WIDTH, DINO_HEIGHT, GREEN);

        // 绘制恐龙的头部（更宽一些）
        draw_rectangle(
            DINO_X - 5.0,
            self.dino_y - 8.0,
            DINO_WIDTH + 12.0,
            15.0,
            GREEN,
        );

        // 绘制恐龙的眼睛
        draw_circle(DINO_X + 8.0, self.dino_y - 2.0, 2.5, BLACK);

        // 绘制恐龙的嘴巴
        draw_line(
            DINO_X + 18.0,
            self.dino_y + 2.0,
            DINO_X + 25.0,
            self.dino_y + 2.0,
            2.0,
            BLACK,
        );

        // 绘制恐龙的尾巴
        draw_rectangle(DINO_X - 5.0, self.dino_y + 15.0, 8.0, 4.0, GREEN);

        // 绘制恐龙的腿（简单的线条）
        if !self.is_jumping {
            let leg_offset = (get_time() * 10.0).sin() as f32 * 2.0;
            // 前腿
            draw_line(
                DINO_X + 8.0,
                self.dino_y + DINO_HEIGHT,
                DINO_X + 8.0,
                self.dino_y + DINO_HEIGHT + 20.0 + leg_offset,
                3.0,
                BLACK,
            );
            // 后腿
            draw_line(
                DINO_X + 15.0,
                self.dino_y + DINO_HEIGHT,
                DINO_X + 15.0,
                self.dino_y + DINO_HEIGHT + 20.0 - leg_offset,
                3.0,
                BLACK,
            );
        }

        // 绘制仙人掌
        for cactus in &self.cacti {
            draw_rectangle(cactus.x, cactus.y, CACTUS_WIDTH, CACTUS_HEIGHT, DARKGREEN);
            // 仙人掌的刺
            for i in 0..3 {
                let spike_y = cactus.y + (i as f32 * 20.0) + 10.0;
                draw_line(cactus.x - 5.0, spike_y, cactus.x, spike_y, 2.0, DARKGREEN);
                draw_line(
                    cactus.x + CACTUS_WIDTH,
                    spike_y,
                    cactus.x + CACTUS_WIDTH + 5.0,
                    spike_y,
                    2.0,
                    DARKGREEN,
                );
            }
        }

        // 绘制五角星
        for star in &self.stars {
            self.draw_star(
                star.x + STAR_SIZE / 2.0,
                star.y + STAR_SIZE / 2.0,
                STAR_SIZE / 2.0,
            );
        }

        // 绘制地面小石子
        for stone in &self.stones {
            // 使用石子自带的颜色
            draw_circle(stone.x, stone.y, stone.size, stone.color);
            
            // 添加高光效果让石子更立体
            let highlight_color = if stone.y > GROUND_Y && stone.y < GROUND_Y2 {
                // 彩色石子使用白色高光
                WHITE
            } else {
                // 灰色石子使用浅灰色高光
                LIGHTGRAY
            };
            
            draw_circle(
                stone.x - stone.size * 0.3, 
                stone.y - stone.size * 0.3, 
                stone.size * 0.3, 
                highlight_color
            );
        }

        // 绘制分数
        let score_text = if self.font.is_some() {
            format!("分数: {}", self.score)
        } else {
            format!("Score: {}", self.score)
        };

        if let Some(font) = &self.font {
            draw_text_ex(
                &score_text,
                20.0,
                30.0,
                TextParams {
                    font: Some(font),
                    font_size: 30,
                    color: BLACK,
                    ..Default::default()
                },
            );
        } else {
            draw_text(&score_text, 20.0, 30.0, 30.0, BLACK);
        }

        // 绘制数学题界面
        if self.showing_math_question {
            self.draw_math_question();
        } else if self.game_over {
            let revive_text = if self.font.is_some() {
                "游戏结束! 按 R 重新开始"
            } else {
                "Game Over! Press R to restart"
            };

            let text_size = if let Some(font) = &self.font {
                measure_text(revive_text, Some(font), 30, 1.0)
            } else {
                measure_text(revive_text, None, 30, 1.0)
            };

            if let Some(font) = &self.font {
                draw_text_ex(
                    revive_text,
                    screen_width() / 2.0 - text_size.width / 2.0,
                    screen_height() / 2.0,
                    TextParams {
                        font: Some(font),
                        font_size: 30,
                        color: RED,
                        ..Default::default()
                    },
                );
            } else {
                draw_text(
                    revive_text,
                    screen_width() / 2.0 - text_size.width / 2.0,
                    screen_height() / 2.0,
                    30.0,
                    RED,
                );
            }
        } else {
            // 绘制操作提示
            let help_text = if self.font.is_some() {
                "按空格键跳跃"
            } else {
                "Press SPACE to jump"
            };

            if let Some(font) = &self.font {
                draw_text_ex(
                    help_text,
                    20.0,
                    screen_height() - 20.0,
                    TextParams {
                        font: Some(font),
                        font_size: 20,
                        color: GRAY,
                        ..Default::default()
                    },
                );
            } else {
                draw_text(help_text, 20.0, screen_height() - 20.0, 20.0, GRAY);
            }
        }
    }

    fn draw_math_question(&self) {
        if let Some(question) = &self.math_question {
            // 绘制半透明背景
            draw_rectangle(
                0.0,
                0.0,
                screen_width(),
                screen_height(),
                Color::new(0.0, 0.0, 0.0, 0.7),
            );

            let center_x = screen_width() / 2.0;
            let center_y = screen_height() / 2.0;

            // 绘制题目背景框
            draw_rectangle(center_x - 200.0, center_y - 150.0, 400.0, 300.0, WHITE);
            draw_rectangle_lines(center_x - 200.0, center_y - 150.0, 400.0, 300.0, 3.0, BLACK);

            let title_text = if self.font.is_some() {
                "答题复活！"
            } else {
                "Revival Challenge!"
            };

            // 绘制标题
            if let Some(font) = &self.font {
                let title_size = measure_text(title_text, Some(font), 30, 1.0);
                draw_text_ex(
                    title_text,
                    center_x - title_size.width / 2.0,
                    center_y - 100.0,
                    TextParams {
                        font: Some(font),
                        font_size: 30,
                        color: BLUE,
                        ..Default::default()
                    },
                );
            } else {
                let title_size = measure_text(title_text, None, 30, 1.0);
                draw_text(
                    title_text,
                    center_x - title_size.width / 2.0,
                    center_y - 100.0,
                    30.0,
                    BLUE,
                );
            }

            // 绘制题目
            let question_size = measure_text(&question.question, None, 40, 1.0);
            draw_text(
                &question.question,
                center_x - question_size.width / 2.0,
                center_y - 40.0,
                40.0,
                BLACK,
            );

            // 绘制输入框边框
            let input_box_x = center_x - 110.0;
            let input_box_y = center_y;
            let input_box_width = 220.0;
            let input_box_height = 40.0;
            draw_rectangle_lines(
                input_box_x,
                input_box_y,
                input_box_width,
                input_box_height,
                2.0,
                GRAY,
            );

            // 绘制输入框
            let input_text = if self.font.is_some() {
                format!("答案: {}", self.input_buffer)
            } else {
                format!("Answer: {}", self.input_buffer)
            };

            // 计算文本高度以实现垂直居中，水平左对齐
            let text_size = if let Some(font) = &self.font {
                measure_text(&input_text, Some(font), 30, 1.0)
            } else {
                measure_text(&input_text, None, 30, 1.0)
            };

            // 水平左对齐（距离左边框10像素），垂直居中
            let text_x = input_box_x + 10.0;
            let text_y = input_box_y + input_box_height / 2.0 + text_size.height / 2.0;

            if let Some(font) = &self.font {
                draw_text_ex(
                    &input_text,
                    text_x,
                    text_y,
                    TextParams {
                        font: Some(font),
                        font_size: 30,
                        color: DARKBLUE,
                        ..Default::default()
                    },
                );
            } else {
                draw_text(&input_text, text_x, text_y, 30.0, DARKBLUE);
            }

            // 绘制提示
            let hint_text = if self.font.is_some() {
                "输入答案后按回车确认，ESC取消"
            } else {
                "Enter answer and press ENTER, ESC to cancel"
            };

            if let Some(font) = &self.font {
                let hint_size = measure_text(hint_text, Some(font), 20, 1.0);
                draw_text_ex(
                    hint_text,
                    center_x - hint_size.width / 2.0,
                    center_y + 80.0,
                    TextParams {
                        font: Some(font),
                        font_size: 20,
                        color: GRAY,
                        ..Default::default()
                    },
                );
            } else {
                let hint_size = measure_text(hint_text, None, 20, 1.0);
                draw_text(
                    hint_text,
                    center_x - hint_size.width / 2.0,
                    center_y + 80.0,
                    20.0,
                    GRAY,
                );
            }
        }
    }

    fn draw_star(&self, center_x: f32, center_y: f32, radius: f32) {
        let points = 5;
        let outer_radius = radius;
        let inner_radius = radius * 0.4;

        let mut vertices = Vec::new();

        for i in 0..(points * 2) {
            let angle = (i as f32 * std::f32::consts::PI) / points as f32;
            let r = if i % 2 == 0 {
                outer_radius
            } else {
                inner_radius
            };
            let x = center_x + r * angle.cos();
            let y = center_y + r * angle.sin();
            vertices.push(Vec2::new(x, y));
        }

        // 绘制五角星的填充
        for i in 1..(vertices.len() - 1) {
            draw_triangle(vertices[0], vertices[i], vertices[i + 1], GOLD);
        }

        // 绘制五角星的边框
        for i in 0..vertices.len() {
            let next = (i + 1) % vertices.len();
            draw_line(
                vertices[i].x,
                vertices[i].y,
                vertices[next].x,
                vertices[next].y,
                2.0,
                ORANGE,
            );
        }
    }
}

#[macroquad::main("Chrome Dino Game")]
async fn main() {
    let mut game_state = GameState::new();
    game_state.load_font().await;

    loop {
        let dt = get_frame_time();

        // 重新开始游戏
        if game_state.game_over && is_key_pressed(KeyCode::R) {
            game_state.reset();
        }

        game_state.update(dt);
        game_state.draw();

        next_frame().await;
    }
}
