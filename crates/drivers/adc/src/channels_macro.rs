#[macro_export]
macro_rules! define_channels_mod {
    ($mod_name:ident, [$($n:literal),*]) => {
        pub mod $mod_name {
            pub struct ConstU<const N: usize>;

            pub trait Channels {}

            $(
                impl Channels for ConstU<$n> {}
            )*
        }
    };
}
