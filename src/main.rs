diff --git a/src/main.rs b/src/main.rs
new file mode 100644
index 0000000000000000000000000000000000000000..98e5bb13a0e784b2b13f23cff1c5195f46b3bd38
--- /dev/null
+++ b/src/main.rs
@@ -0,0 +1,631 @@
+//! Prototype Fortune Street (Itadaki Street) board game using Bevy.
+//! The implementation follows the Wii "Fortune Street" flow: players roll dice,
+//! move along a looping path of shops, collect suits (spade/heart/diamond/club),
+//! visit the bank to level up and receive salary, pay shop fees, invest in stocks
+//! for districts, and can upgrade shops they own. This prototype focuses on a 2D
+//! UI that visualizes the board, players, and key menus.
+
+use bevy::{input::mouse::MouseWheel, prelude::*};
+use rand::Rng;
+use std::collections::{HashMap, HashSet};
+
+const TILE_SIZE: f32 = 48.0;
+const BOARD_COLOR: Color = Color::rgb(0.15, 0.15, 0.25);
+const BANK_COLOR: Color = Color::rgb(0.9, 0.8, 0.25);
+const PROPERTY_COLOR: Color = Color::rgb(0.25, 0.7, 0.45);
+const SUIT_COLOR: Color = Color::rgb(0.6, 0.25, 0.6);
+const CHANCE_COLOR: Color = Color::rgb(0.25, 0.55, 0.9);
+
+fn main() {
+    App::new()
+        .add_plugins(DefaultPlugins.set(WindowPlugin {
+            primary_window: Some(Window {
+                title: "Itadaki Street Prototype".to_string(),
+                resolution: (1280.0, 720.0).into(),
+                resizable: true,
+                ..Default::default()
+            }),
+            ..Default::default()
+        }))
+        .insert_resource(Game::new())
+        .insert_resource(UiState::default())
+        .insert_resource(TurnTimer(Timer::from_seconds(2.0, TimerMode::Repeating)))
+        .add_systems(Startup, (setup_camera, setup_board, setup_ui))
+        .add_systems(Update, (camera_controls, update_ui, toggle_menu, bot_turns))
+        .run();
+}
+
+#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
+enum Suit {
+    Spade,
+    Heart,
+    Diamond,
+    Club,
+}
+
+impl Suit {
+    fn icon(&self) -> &'static str {
+        match self {
+            Suit::Spade => "\u{2660}",
+            Suit::Heart => "\u{2665}",
+            Suit::Diamond => "\u{2666}",
+            Suit::Club => "\u{2663}",
+        }
+    }
+}
+
+#[derive(Debug, Clone)]
+enum TileKind {
+    Bank,
+    Property {
+        district: &'static str,
+        price: i32,
+        base_fee: i32,
+    },
+    Suit(Suit),
+    Chance,
+}
+
+#[derive(Debug, Clone)]
+struct Tile {
+    index: usize,
+    position: Vec2,
+    kind: TileKind,
+}
+
+#[derive(Debug, Clone, Copy, PartialEq, Eq)]
+enum PlayerKind {
+    Human,
+    Bot,
+}
+
+impl Default for PlayerKind {
+    fn default() -> Self {
+        PlayerKind::Human
+    }
+}
+
+#[derive(Debug, Default, Clone)]
+struct PlayerState {
+    name: String,
+    kind: PlayerKind,
+    cash: i32,
+    stocks: HashMap<&'static str, i32>,
+    properties: HashSet<usize>,
+    suits: HashSet<Suit>,
+    position: usize,
+    level: u32,
+}
+
+impl PlayerState {
+    fn net_worth(&self, board: &[Tile]) -> i32 {
+        let property_value: i32 = self
+            .properties
+            .iter()
+            .filter_map(|index| match &board[*index].kind {
+                TileKind::Property { price, .. } => Some(*price),
+                _ => None,
+            })
+            .sum();
+        let stock_value: i32 = self.stocks.values().sum();
+        self.cash + property_value + stock_value
+    }
+}
+
+#[derive(Resource)]
+struct Game {
+    board: Vec<Tile>,
+    players: Vec<PlayerState>,
+    current_turn: usize,
+    district_shop_count: HashMap<&'static str, usize>,
+}
+
+impl Game {
+    fn new() -> Self {
+        let board = generate_board();
+        let players = vec![
+            PlayerState {
+                name: "Hero".into(),
+                kind: PlayerKind::Human,
+                cash: 2500,
+                ..Default::default()
+            },
+            PlayerState {
+                name: "Bot A".into(),
+                kind: PlayerKind::Bot,
+                cash: 2500,
+                ..Default::default()
+            },
+            PlayerState {
+                name: "Bot B".into(),
+                kind: PlayerKind::Bot,
+                cash: 2500,
+                ..Default::default()
+            },
+        ];
+        Self {
+            board,
+            players,
+            current_turn: 0,
+            district_shop_count: HashMap::new(),
+        }
+    }
+}
+
+#[allow(dead_code)]
+#[derive(Component)]
+struct TileEntity(usize);
+
+#[derive(Component)]
+struct PlayerToken(usize);
+
+#[derive(Resource, Default)]
+struct UiState {
+    menu_open: bool,
+    stocks_open: bool,
+}
+
+#[derive(Resource)]
+struct TurnTimer(Timer);
+
+fn setup_camera(mut commands: Commands) {
+    commands.spawn(Camera2dBundle {
+        transform: Transform::from_xyz(0.0, 0.0, 999.0),
+        projection: OrthographicProjection {
+            scale: 1.0,
+            ..Default::default()
+        },
+        ..Default::default()
+    });
+}
+
+fn setup_board(mut commands: Commands, game: Res<Game>) {
+    for tile in &game.board {
+        let (color, label) = match &tile.kind {
+            TileKind::Bank => (BANK_COLOR, "Bank".to_string()),
+            TileKind::Property { district, .. } => (PROPERTY_COLOR, (*district).to_string()),
+            TileKind::Suit(suit) => (SUIT_COLOR, format!("{} Suit", suit.icon())),
+            TileKind::Chance => (CHANCE_COLOR, "Chance".to_string()),
+        };
+
+        commands
+            .spawn(SpriteBundle {
+                sprite: Sprite {
+                    color,
+                    custom_size: Some(Vec2::splat(TILE_SIZE)),
+                    ..Default::default()
+                },
+                transform: Transform::from_translation(tile.position.extend(0.0)),
+                ..Default::default()
+            })
+            .insert(TileEntity(tile.index))
+            .with_children(|parent| {
+                parent.spawn(Text2dBundle {
+                    text: Text::from_section(
+                        label.clone(),
+                        TextStyle {
+                            font_size: 14.0,
+                            color: Color::WHITE,
+                            ..Default::default()
+                        },
+                    ),
+                    transform: Transform::from_xyz(0.0, 0.0, 1.0),
+                    ..Default::default()
+                });
+            });
+    }
+
+    for (idx, player) in game.players.iter().enumerate() {
+        let offset = (idx as f32 - 1.0) * 12.0;
+        let position = game.board[player.position].position + Vec2::new(offset, offset);
+        commands
+            .spawn(SpriteBundle {
+                sprite: Sprite {
+                    color: Color::rgb(0.9 - 0.2 * idx as f32, 0.2, 0.9),
+                    custom_size: Some(Vec2::splat(20.0)),
+                    ..Default::default()
+                },
+                transform: Transform::from_translation(position.extend(2.0)),
+                ..Default::default()
+            })
+            .insert(PlayerToken(idx));
+    }
+}
+
+#[derive(Component)]
+struct UiRoot;
+
+#[derive(Component)]
+struct InfoText;
+
+#[derive(Component)]
+struct MenuPanel;
+
+#[derive(Component)]
+struct StockPanel;
+
+fn setup_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
+    let font = asset_server.load("fonts/FiraSans-Bold.ttf");
+    commands
+        .spawn((NodeBundle {
+            style: Style {
+                width: Val::Percent(100.0),
+                height: Val::Percent(100.0),
+                padding: UiRect::all(Val::Px(12.0)),
+                ..Default::default()
+            },
+            background_color: BackgroundColor(Color::NONE),
+            ..Default::default()
+        }, UiRoot))
+        .with_children(|parent| {
+            parent
+                .spawn(NodeBundle {
+                    style: Style {
+                        width: Val::Percent(30.0),
+                        height: Val::Percent(100.0),
+                        flex_direction: FlexDirection::Column,
+                        row_gap: Val::Px(8.0),
+                        ..Default::default()
+                    },
+                    background_color: BackgroundColor(BOARD_COLOR.with_a(0.5)),
+                    ..Default::default()
+                })
+                .with_children(|sidebar| {
+                    sidebar.spawn((TextBundle {
+                        text: Text::from_section(
+                            "Turn info will appear here",
+                            TextStyle {
+                                font: font.clone(),
+                                font_size: 18.0,
+                                color: Color::WHITE,
+                            },
+                        ),
+                        ..Default::default()
+                    }, InfoText));
+                });
+
+            parent
+                .spawn((
+                    NodeBundle {
+                        style: Style {
+                            position_type: PositionType::Absolute,
+                            right: Val::Px(12.0),
+                            bottom: Val::Px(12.0),
+                            width: Val::Px(320.0),
+                            height: Val::Px(280.0),
+                            display: Display::None,
+                            flex_direction: FlexDirection::Column,
+                            padding: UiRect::all(Val::Px(8.0)),
+                            row_gap: Val::Px(8.0),
+                            ..Default::default()
+                        },
+                        background_color: BackgroundColor(Color::rgb(0.1, 0.1, 0.15)),
+                        ..Default::default()
+                    },
+                    MenuPanel,
+                ))
+                .with_children(|menu| {
+                    menu.spawn(TextBundle::from_section(
+                        "Main Menu\n- Buy/Upgrade Shops\n- Trade\n- Stock Market (press S)\n- Fast decision toggles",
+                        TextStyle {
+                            font: font.clone(),
+                            font_size: 16.0,
+                            color: Color::WHITE,
+                        },
+                    ));
+                });
+
+            parent
+                .spawn((
+                    NodeBundle {
+                        style: Style {
+                            position_type: PositionType::Absolute,
+                            left: Val::Px(12.0),
+                            bottom: Val::Px(12.0),
+                            width: Val::Px(360.0),
+                            height: Val::Px(260.0),
+                            display: Display::None,
+                            flex_direction: FlexDirection::Column,
+                            padding: UiRect::all(Val::Px(8.0)),
+                            row_gap: Val::Px(6.0),
+                            ..Default::default()
+                        },
+                        background_color: BackgroundColor(Color::rgb(0.12, 0.1, 0.16)),
+                        ..Default::default()
+                    },
+                    StockPanel,
+                ))
+                .with_children(|stock| {
+                    stock.spawn(TextBundle::from_section(
+                        "Stocks Menu\nUse +/- to adjust bids per district, confirm to purchase/sell.",
+                        TextStyle {
+                            font: font.clone(),
+                            font_size: 16.0,
+                            color: Color::WHITE,
+                        },
+                    ));
+                });
+        });
+}
+
+fn camera_controls(
+    keyboard: Res<ButtonInput<KeyCode>>,
+    mut scroll_evr: EventReader<MouseWheel>,
+    mut query: Query<(&mut Transform, &mut OrthographicProjection), With<Camera2d>>,
+    time: Res<Time>,
+) {
+    for (mut transform, mut projection) in query.iter_mut() {
+        let mut direction = Vec3::ZERO;
+        if keyboard.pressed(KeyCode::ArrowLeft) || keyboard.pressed(KeyCode::KeyA) {
+            direction.x -= 1.0;
+        }
+        if keyboard.pressed(KeyCode::ArrowRight) || keyboard.pressed(KeyCode::KeyD) {
+            direction.x += 1.0;
+        }
+        if keyboard.pressed(KeyCode::ArrowUp) || keyboard.pressed(KeyCode::KeyW) {
+            direction.y += 1.0;
+        }
+        if keyboard.pressed(KeyCode::ArrowDown) || keyboard.pressed(KeyCode::KeyS) {
+            direction.y -= 1.0;
+        }
+        let speed = 400.0 * time.delta_seconds();
+        transform.translation += direction.normalize_or_zero() * speed;
+
+        for ev in scroll_evr.read() {
+            projection.scale = (projection.scale * (1.0 - ev.y * 0.1)).clamp(0.5, 2.5);
+        }
+    }
+}
+
+fn toggle_menu(
+    keyboard: Res<ButtonInput<KeyCode>>,
+    mut ui_state: ResMut<UiState>,
+    mut menus: Query<&mut Style, With<MenuPanel>>,
+    mut stocks: Query<&mut Style, (With<StockPanel>, Without<MenuPanel>)>,
+) {
+    if keyboard.just_pressed(KeyCode::KeyM) {
+        ui_state.menu_open = !ui_state.menu_open;
+    }
+    if keyboard.just_pressed(KeyCode::KeyS) {
+        ui_state.stocks_open = !ui_state.stocks_open;
+        ui_state.menu_open = ui_state.menu_open || ui_state.stocks_open;
+    }
+
+    for mut style in menus.iter_mut() {
+        style.display = if ui_state.menu_open {
+            Display::Flex
+        } else {
+            Display::None
+        };
+    }
+    for mut style in stocks.iter_mut() {
+        style.display = if ui_state.stocks_open {
+            Display::Flex
+        } else {
+            Display::None
+        };
+    }
+}
+
+fn bot_turns(
+    time: Res<Time>,
+    mut timer: ResMut<TurnTimer>,
+    mut game: ResMut<Game>,
+    mut tokens: Query<(&mut Transform, &PlayerToken)>,
+) {
+    if !timer.0.tick(time.delta()).just_finished() {
+        return;
+    }
+
+    if game.players.is_empty() {
+        return;
+    }
+
+    let current = game.current_turn % game.players.len();
+    let is_bot = matches!(game.players[current].kind, PlayerKind::Bot);
+    if !is_bot {
+        game.current_turn = (game.current_turn + 1) % game.players.len();
+        return;
+    }
+
+    let roll = rand::thread_rng().gen_range(1..=6);
+    advance_player(current, roll, &mut game, &mut tokens);
+    game.current_turn = (game.current_turn + 1) % game.players.len();
+}
+
+fn advance_player(
+    player_idx: usize,
+    roll: i32,
+    game: &mut Game,
+    tokens: &mut Query<(&mut Transform, &PlayerToken)>,
+) {
+    let board_len = game.board.len();
+    {
+        let player = &mut game.players[player_idx];
+        player.position = ((player.position as i32 + roll) as usize) % board_len;
+    }
+
+    let tile_index = game.players[player_idx].position;
+    let tile_kind = game.board[tile_index].kind.clone();
+    let tile_position = game.board[tile_index].position;
+
+    handle_tile(tile_index, &tile_kind, player_idx, game);
+
+    for (mut transform, token) in tokens.iter_mut() {
+        if token.0 == player_idx {
+            transform.translation = tile_position.extend(2.0);
+        }
+    }
+}
+
+fn handle_tile(tile_index: usize, kind: &TileKind, player_idx: usize, game: &mut Game) {
+    match kind {
+        TileKind::Bank => {
+            let player = &mut game.players[player_idx];
+            if player.suits.len() == 4 {
+                player.level += 1;
+                let salary = 500 + (player.net_worth(&game.board) as f32 * 0.1) as i32;
+                player.cash += salary;
+                player.suits.clear();
+            }
+        }
+        TileKind::Property {
+            district,
+            price,
+            base_fee,
+        } => {
+            let owner = game
+                .players
+                .iter()
+                .enumerate()
+                .find(|(_, p)| p.properties.contains(&tile_index));
+            match owner {
+                Some((owner_idx, _)) if owner_idx != player_idx => {
+                    let fee = *base_fee;
+                    let payer = &mut game.players[player_idx];
+                    payer.cash -= fee;
+                    let receiver = &mut game.players[owner_idx];
+                    receiver.cash += fee;
+                }
+                None => {
+                    let buyer = &mut game.players[player_idx];
+                    if buyer.cash >= *price {
+                        buyer.cash -= *price;
+                        buyer.properties.insert(tile_index);
+                        *game.district_shop_count.entry(district).or_default() += 1;
+                    }
+                }
+                _ => {}
+            }
+        }
+        TileKind::Suit(suit) => {
+            game.players[player_idx].suits.insert(*suit);
+        }
+        TileKind::Chance => {
+            let delta = rand::thread_rng().gen_range(-150..=200);
+            game.players[player_idx].cash += delta;
+        }
+    }
+}
+
+fn update_ui(mut info_text: Query<&mut Text, With<InfoText>>, game: Res<Game>) {
+    if let Ok(mut text) = info_text.get_single_mut() {
+        let mut content = String::new();
+        content.push_str("Fortune Street Loop\nRoll dice to move, buy shops, collect suits, and level up at the bank.\n\n");
+        content.push_str(&format!(
+            "Current turn: {}\n\n",
+            game.players[game.current_turn].name
+        ));
+        for (idx, player) in game.players.iter().enumerate() {
+            let suits: String = [Suit::Spade, Suit::Heart, Suit::Diamond, Suit::Club]
+                .iter()
+                .map(|s| {
+                    if player.suits.contains(s) {
+                        s.icon()
+                    } else {
+                        "_"
+                    }
+                })
+                .collect();
+            content.push_str(&format!(
+                "{} [{}] \nCash: {} | Net: {} | Level: {}\nSuits: {}\nProperties: {}\nStocks: {:?}\n\n",
+                player.name,
+                match player.kind {
+                    PlayerKind::Human => "Human",
+                    PlayerKind::Bot => "Bot",
+                },
+                player.cash,
+                player.net_worth(&game.board),
+                player.level,
+                suits,
+                player.properties.len(),
+                player.stocks
+            ));
+            if idx == game.current_turn {
+                content.push_str("-- taking turn --\n\n");
+            }
+        }
+        text.sections[0].value = content;
+    }
+}
+
+fn generate_board() -> Vec<Tile> {
+    let mut tiles = Vec::new();
+    // Square loop 4x4 path with an inner bank.
+    let layout = vec![
+        TileKind::Bank,
+        TileKind::Property {
+            district: "Downtown",
+            price: 300,
+            base_fee: 80,
+        },
+        TileKind::Suit(Suit::Spade),
+        TileKind::Property {
+            district: "Downtown",
+            price: 320,
+            base_fee: 90,
+        },
+        TileKind::Chance,
+        TileKind::Property {
+            district: "Plaza",
+            price: 280,
+            base_fee: 75,
+        },
+        TileKind::Suit(Suit::Heart),
+        TileKind::Property {
+            district: "Plaza",
+            price: 260,
+            base_fee: 70,
+        },
+        TileKind::Chance,
+        TileKind::Property {
+            district: "Harbor",
+            price: 350,
+            base_fee: 95,
+        },
+        TileKind::Suit(Suit::Diamond),
+        TileKind::Property {
+            district: "Harbor",
+            price: 360,
+            base_fee: 105,
+        },
+        TileKind::Chance,
+        TileKind::Property {
+            district: "Grove",
+            price: 240,
+            base_fee: 60,
+        },
+        TileKind::Suit(Suit::Club),
+        TileKind::Property {
+            district: "Grove",
+            price: 260,
+            base_fee: 65,
+        },
+        TileKind::Chance,
+    ];
+
+    // Lay tiles on a rough square track.
+    let mut coords = Vec::new();
+    for x in 0..4 {
+        coords.push(Vec2::new(x as f32 * TILE_SIZE, 0.0));
+    }
+    for y in 1..4 {
+        coords.push(Vec2::new(3.0 * TILE_SIZE, y as f32 * TILE_SIZE));
+    }
+    for x in (0..3).rev() {
+        coords.push(Vec2::new(x as f32 * TILE_SIZE, 3.0 * TILE_SIZE));
+    }
+    for y in (1..3).rev() {
+        coords.push(Vec2::new(0.0, y as f32 * TILE_SIZE));
+    }
+
+    for (index, (kind, pos)) in layout.into_iter().zip(coords.into_iter()).enumerate() {
+        tiles.push(Tile {`
+            index,
+            position: pos - Vec2::splat(1.5 * TILE_SIZE),
+            kind,
+        });
+    }
+
+    tiles
+}
