
#![allow(unused)] //silence unused warning while learning


use std::fs::File;
use std::fs;
use rand::thread_rng;
use rand::seq::SliceRandom;
use std::path::Path;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use bevy::prelude::*;
use bevy::app::AppExit;
use bevy::input::keyboard::KeyboardInput;

use serde::{Deserialize, Serialize};

const PANEL_SPRITE: &str="abbasid_panel.png";
const MENU1_KEYS: [char;4] = ['Q','W','E','R'];

// EntitY Component System Resource

// Begin Resources

struct Quizz{
    building: Building,
    solved: bool,
}

#[derive(Debug, Deserialize, Serialize,Clone)]
struct Building {
    fname: String,
    age: usize,
    key: char,
}

struct Buildings {
    buildings: Vec<Building>
}

pub struct MenuState {
    age:usize,
}

pub struct Materials {
    materials: Handle<ColorMaterial>,
}

struct WinSize{
    w: f32,
    h: f32,
}


//  End Resources



// #[wasm_bindgen]
fn main() {
    // When building for WASM, print panics to the browser console
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();

    let mut app = App::build();
    
    app.add_plugins_with(DefaultPlugins, |group| { group.disable::<bevy::log::LogPlugin>()});

    // when building for Web, use WebGL2 rendering
    #[cfg(target_arch = "wasm32")]
    app.add_plugin(bevy_webgl2::WebGL2Plugin);
    


    app.insert_resource(ClearColor(Color::rgb(0., 0., 0.)));
    app.insert_resource(WindowDescriptor {
        title: "AMAVillager".to_string(),
        width: 600.0,
        height: 600.0,
        ..Default::default()
    });
    app.add_plugins(DefaultPlugins );
    app.add_startup_system(setup.system());
    app.add_startup_stage("", SystemStage::single(spawn_quizzitem.system()));
    app.add_startup_stage("game_setup_actors", SystemStage::single(panel_spawn.system()));
    app.add_system(handle_quizz_keypresses.system());
    app.add_system(quizz_logic.system());
    app.add_system(quit.system());



    app.run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut windows: ResMut<Windows>
){

    //========== BEGIN RESOURCES

    // WinSize
    let window = windows.get_primary_mut().unwrap();
    commands.insert_resource(WinSize {
        w: window.width(),
        h: window.height()
    });
    
    // Buildings
    let buildings: Vec<Building> = serde_json::from_reader(File::open("src/data.json").unwrap()).expect("Error while reading or parsing!") ;
    commands.insert_resource(Buildings{buildings});

    // Materials
    commands.insert_resource(Materials {
        materials: materials.add(asset_server.load(PANEL_SPRITE).into()),
    });

    // MenuState
    commands.insert_resource(MenuState{
        age:0,
    });
    //============ END RESOURCES

    // camera
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    
    //position window
    window.set_position(IVec2::new(0, 0));
}


fn panel_spawn( mut commands: Commands, materials: Res<Materials>, win_size: Res<WinSize>) {
    let bottom= - win_size.h / 2. ;
    commands.spawn_bundle(SpriteBundle {
        material: materials.materials.clone(),
        transform: Transform {
            translation: Vec3::new(0., bottom + 218. / 2. + 5., 10. ),
            ..Default::default()
        },
        ..Default::default()
    });
}


fn spawn_quizzitem(mut commands: Commands,mut materials: ResMut<Assets<ColorMaterial>>, asset_server: Res<AssetServer>, buildings: Res<Buildings>){

    let mut rng= thread_rng();
    let building =buildings.buildings.choose(&mut rng).unwrap();
    let fname =  "buildings/".to_owned() + &building.fname;
    
    let quizz_material= materials.add(asset_server.load(fname.as_str()).into());
    let building=Building{fname:building.fname.clone(), key:building.key, age:building.age};
    let solved= false;

    commands.spawn_bundle(SpriteBundle {
            material: quizz_material.clone(),
            transform: Transform {
                translation: Vec3::new(0., 0., 10. ),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Quizz{ 
            building,
            solved,
        });
} 

fn quizz_logic( mut query: Query<(Entity, &mut Quizz), With<Quizz>>,
                // mut exit: EventWriter<AppExit>,
                mut commands: Commands,
                mut materials: ResMut<Assets<ColorMaterial>>,
                asset_server: Res<AssetServer>,
                buildings: Res<Buildings>
        ){
            if  let Ok((e, mut quizz)) = query.single_mut()  {
                if (quizz.solved == true ){
                    //remove current quizz entity ,spanw a new one
                    commands.entity(e).despawn();
                    quizz.solved=false;
                    spawn_quizzitem(commands, materials, asset_server, buildings);
                }
            }
}


fn handle_quizz_keypresses(keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&mut Quizz), With<Quizz>>,
    mut key_evr: EventReader<KeyboardInput>,
    mut menu_state: ResMut<MenuState>){
    use bevy::input::ElementState;
    
    if let Ok( mut quizz ) = query.single_mut() {
        let mut building= quizz.building.clone();
        let mut solved= quizz.solved;
        for ev in key_evr.iter() {
            match ev.state {
                ElementState::Pressed => {
                    // println!("Key press: {:?} ({})", ev.key_code, ev.scan_code)
                    let valid_keys=[KeyCode::Q, KeyCode::W, KeyCode::E, KeyCode::R ];
                    // Press 'Esc' key to reset input
                    if( ev.key_code.unwrap_or(KeyCode::Compose) == KeyCode::Escape ){
                        menu_state.age=0;    
                        println!("Reset")
                    }
                    // Age Selection
                    else if( valid_keys.contains(&ev.key_code.unwrap_or(KeyCode::Compose))
                        && menu_state.age == 0 ) {
                            
                            menu_state.age= match ev.key_code.unwrap(){
                                KeyCode::Q => 1,
                                KeyCode::W => 2,
                                KeyCode::E => 3,
                                KeyCode::R => 4,
                                _ => 0,
                            };

                            building.age;
                            println!("Age Select {}", menu_state.age);
                    }
                    // Building Selection 
                    else if( menu_state.age == building.age
                             && ev.key_code.unwrap_or(KeyCode::Compose) == char2keycode(building.key).unwrap() ){
                        
                            println!("BINGO!");
                            quizz.solved=true;
                            menu_state.age=0;
                    }
                    else{
                        println!("Nope! Retry")
                    }

                }
                ElementState::Released => {
                    // println!("Key release: {:?} ({})", ev.key_code, ev.scan_code);
                }
            }
        }
    }
}

fn quit(
    keys: Res<Input<KeyCode>>,
    mut exit: EventWriter<AppExit>
) {
    if keys.pressed(KeyCode::LControl) && keys.pressed(KeyCode::C) {
        println!("Exiting...");
        exit.send(AppExit);
    }
}

// Utility

fn char2keycode( input: char) -> Result<KeyCode, ()>{
    match input {
        'Q' => Ok(KeyCode::Q),
        'W' => Ok(KeyCode::W),
        'E' => Ok(KeyCode::E),
        'R' => Ok(KeyCode::R),
        'A' => Ok(KeyCode::A),
        'S' => Ok(KeyCode::S),
        'D' => Ok(KeyCode::D),
        'Z' => Ok(KeyCode::Z),
        'X' => Ok(KeyCode::X),
        'C' => Ok(KeyCode::C),
        _ => Err(()),
    }
}
