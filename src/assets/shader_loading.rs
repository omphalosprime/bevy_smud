use bevy::{prelude::*, reflect::TypeUuid};

pub const PRELUDE_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 11291576006157771079);
pub const PRELUDE_SHADER_IMPORT: &str = "bevy_smud::prelude";

pub const SHAPES_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 10055894596049459186);
pub const SHAPES_SHADER_IMPORT: &str = "bevy_smud::shapes";

pub const COLORIZE_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 10050447940405429418);
pub const COLORIZE_SHADER_IMPORT: &str = "bevy_smud::colorize";

pub const SMUD_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 5645555317811706725);
pub const SMUD_SHADER_IMPORT: &str = "bevy_smud::smud";

pub const VERTEX_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 16846632126033267571);
pub const VERTEX_SHADER_IMPORT: &str = "bevy_smud::vertex";

pub const FRAGMENT_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 10370213491934870425);
pub const FRAGMENT_SHADER_IMPORT: &str = "bevy_smud::fragment";

pub const DEFAULT_FILL_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 18184663565780163454);
pub const DEFAULT_FILL_IMPORT: &str = "bevy_smud::default_fill";

pub const SIMPLE_FILL_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 16286090377316294491);
pub const SIMPLE_FILL_IMPORT: &str = "bevy_smud::simple_fill";

// unused:
// 16950619110804285379
// 4146091551367169642
// 8080191226000727371
// 17031499878237077924
// 17982773815777006860
// 1530570659737977289

#[cfg(feature = "smud_shader_hot_reloading")]
struct HotShader {
    strong_handle: Handle<Shader>,
    untyped_handle: Option<HandleUntyped>,
    loaded: bool,
    import_path: String,
}

// Needed to keep the shaders alive
#[cfg(feature = "smud_shader_hot_reloading")]
struct HotShaders<T> {
    shaders: Vec<HotShader>,
    marker: std::marker::PhantomData<T>,
}

#[cfg(feature = "smud_shader_hot_reloading")]
impl<T> Default for HotShaders<T> {
    fn default() -> Self {
        Self {
            shaders: Default::default(),
            marker: Default::default(),
        }
    }
}

#[cfg(feature = "smud_shader_hot_reloading")]
fn setup_shader_imports<T: 'static + Send + Sync>(
    mut hot_shaders: ResMut<HotShaders<T>>,
    mut shaders: ResMut<Assets<Shader>>,
    asset_server: Res<AssetServer>,
) {
    for hot_shader in hot_shaders.shaders.iter_mut() {
        if !hot_shader.loaded
            && asset_server.get_load_state(hot_shader.strong_handle.clone())
                == bevy::asset::LoadState::Loaded
        {
            shaders
                .get_mut(hot_shader.strong_handle.clone())
                .unwrap()
                .set_import_path(&hot_shader.import_path);

            hot_shader.loaded = true;
        }
    }
}

