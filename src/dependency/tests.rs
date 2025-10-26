use super::*;
use pastey::paste;

macro_rules! test_function {
    (
        $ident:ident,
        $out:expr$(,)?
    ) => {
        #[test]
        fn $ident() {
            let Inputs(input, dependency) = Inputs::$ident();

            assert_eq!(
                Dependency::from_str(&input).expect("failed to parse dependency"),
                dependency,
            );
            assert_eq!(dependency.to_string(), $out, "failed to format dependency");
        }
    };
    (
        $ident:ident,
        $modifier:ident,
        $out:expr$(,)?
    ) => {
        #[test]
        fn $ident() {
            let Inputs(input, dependency) = Inputs::$ident().$modifier();

            assert_eq!(
                Dependency::from_str(&input).expect("failed to parse dependency"),
                dependency,
            );
            assert_eq!(dependency.to_string(), $out, "failed to format dependency");
        }
    };
}

macro_rules! test_module {
    (
        $http_repo_out:expr,
        $http_repo_no_extension_out:expr,
        $ssh_repo_out:expr,
        $ssh_repo_no_extension_out:expr$(,)?
    ) => {
        mod base {
            use super::*;

            test_function!(http_repo, $http_repo_out);
            test_function!(http_repo_no_extension, $http_repo_no_extension_out);
            test_function!(ssh_repo, $ssh_repo_out);
            test_function!(ssh_repo_no_extension, $ssh_repo_no_extension_out);
        }
    };
    (
        $crates_io_out:expr,
        $http_repo_out:expr,
        $http_repo_no_extension_out:expr,
        $ssh_repo_out:expr,
        $ssh_repo_no_extension_out:expr$(,)?
    ) => {
        mod base {
            use super::*;

            test_function!(crates_io, $crates_io_out);
            test_function!(http_repo, $http_repo_out);
            test_function!(http_repo_no_extension, $http_repo_no_extension_out);
            test_function!(ssh_repo_no_extension, $ssh_repo_no_extension_out);
        }
    };
    (
        $ident:ident,
        $modifier:expr,
        $http_repo_out:expr,
        $http_repo_no_extension_out:expr,
        $ssh_repo_out:expr,
        $ssh_repo_no_extension_out:expr$(,)?
    ) => {
        mod $ident {
            use super::*;

            impl Inputs {
                pub fn $ident(self) -> Self {
                    $modifier(self)
                }
            }

            test_function!(http_repo, $ident, $http_repo_out);
            test_function!(http_repo_no_extension, $ident, $http_repo_no_extension_out);
            test_function!(ssh_repo, $ident, $ssh_repo_out);
            test_function!(ssh_repo_no_extension, $ident, $ssh_repo_no_extension_out);
        }
    };
    (
        $ident:ident,
        $modifier:expr,
        $crates_io_out:expr,
        $http_repo_out:expr,
        $http_repo_no_extension_out:expr,
        $ssh_repo_out:expr,
        $ssh_repo_no_extension_out:expr$(,)?
    ) => {
        mod $ident {
            use super::*;

            impl Inputs {
                pub fn $ident(self) -> Self {
                    $modifier(self)
                }
            }

            test_function!(crates_io, $ident, $crates_io_out);
            test_function!(http_repo, $ident, $http_repo_out);
            test_function!(http_repo_no_extension, $ident, $http_repo_no_extension_out);
            test_function!(ssh_repo, $ident, $ssh_repo_out);
            test_function!(ssh_repo_no_extension, $ident, $ssh_repo_no_extension_out);
        }
    };
}

macro_rules! test_module_extended {
    (
        $ident:ident,
        ($($modifier:ident),+ $(,)?),
        $http_repo_out:expr,
        $http_repo_no_extension_out:expr,
        $ssh_repo_out:expr,
        $ssh_repo_no_extension_out:expr,
    ) => {
        mod $ident {
            use super::*;

            impl Inputs {
                fn $ident(self) -> Self {
                    self$(.$modifier())+
                }
            }

            test_function!(http_repo, $ident, $http_repo_out);
            test_function!(
                http_repo_no_extension,
                $ident,
                $http_repo_no_extension_out,
            );
            test_function!(ssh_repo, $ident, $ssh_repo_out);
            test_function!(
                ssh_repo_no_extension,
                $ident,
                $ssh_repo_no_extension_out,
            );
        }
    };
    (
        $ident:ident,
        ($($modifier:ident),+ $(,)?),
        $crates_io_out:expr,
        $http_repo_out:expr,
        $http_repo_no_extension_out:expr,
        $ssh_repo_out:expr,
        $ssh_repo_no_extension_out:expr$(,)?,
    ) => {
        mod $ident {
            use super::*;

            impl Inputs {
                fn $ident(self) -> Self {
                    self$(.$modifier())+
                }
            }

            test_function!(crates_io, $ident, $crates_io_out);
            test_function!(http_repo, $ident, $http_repo_out);
            test_function!(
                http_repo_no_extension,
                $ident,
                $http_repo_no_extension_out,
            );
            test_function!(ssh_repo, $ident, $ssh_repo_out);
            test_function!(
                ssh_repo_no_extension,
                $ident,
                $ssh_repo_no_extension_out,
            );
        }
    };
}

macro_rules! test_modules {
    (
        ($first:ident, $second:ident),
        $http_repo_out:expr,
        $http_repo_no_extension_out:expr,
        $ssh_repo_out:expr,
        $ssh_repo_no_extension_out:expr,
    ) => {
        paste! {
            test_module_extended!(
                [<$first _and_ $second>],
                ($first, $second),
                $http_repo_out,
                $http_repo_no_extension_out,
                $ssh_repo_out,
                $ssh_repo_no_extension_out,
            );
            /*
            test_module_extended!(
                [<$second _and_ $first>],
                ($second, $first),
                $http_repo_out,
                $http_repo_no_extension_out,
                $ssh_repo_out,
                $ssh_repo_no_extension_out,
            );
            */
        }
    };
    (
        ($first:ident, $second:ident),
        $crates_io_out:expr,
        $http_repo_out:expr,
        $http_repo_no_extension_out:expr,
        $ssh_repo_out:expr,
        $ssh_repo_no_extension_out:expr$(,)?,
    ) => {
        paste! {
            test_module_extended!(
                [<$first _and_ $second>],
                ($first, $second),
                $crates_io_out,
                $http_repo_out,
                $http_repo_no_extension_out,
                $ssh_repo_out,
                $ssh_repo_no_extension_out,
            );
            /*
            test_module_extended!(
                [<$second _and_ $first>],
                ($second, $first),
                $crates_io_out,
                $http_repo_out,
                $http_repo_no_extension_out,
                $ssh_repo_out,
                $ssh_repo_no_extension_out,
            );
            */
        }
    };
    (
        ($first:ident, $second:ident, $third:ident),
        $http_repo_out:expr,
        $http_repo_no_extension_out:expr,
        $ssh_repo_out:expr,
        $ssh_repo_no_extension_out:expr,
    ) => {
        paste! {
            test_module_extended!(
                [<$first _ $second _and_ $third>],
                ($first, $second, $third),
                $http_repo_out,
                $http_repo_no_extension_out,
                $ssh_repo_out,
                $ssh_repo_no_extension_out,
            );
            /*
            test_module_extended!(
                [<$second _ $first _and_ $third>],
                ($second, $first, $third),
                $http_repo_out,
                $http_repo_no_extension_out,
                $ssh_repo_out,
                $ssh_repo_no_extension_out,
            );
            test_module_extended!(
                [<$third _ $first _and_ $second>],
                ($third, $first, $second),
                $http_repo_out,
                $http_repo_no_extension_out,
                $ssh_repo_out,
                $ssh_repo_no_extension_out,
            );
            test_module_extended!(
                [<$first _ $third _and_ $second>],
                ($first, $third, $second),
                $http_repo_out,
                $http_repo_no_extension_out,
                $ssh_repo_out,
                $ssh_repo_no_extension_out,
            );
            test_module_extended!(
                [<$second _ $third _and_ $first>],
                ($second, $third, $first),
                $http_repo_out,
                $http_repo_no_extension_out,
                $ssh_repo_out,
                $ssh_repo_no_extension_out,
            );
            test_module_extended!(
                [<$third _ $second _and_ $first>],
                ($third, $second, $first),
                $http_repo_out,
                $http_repo_no_extension_out,
                $ssh_repo_out,
                $ssh_repo_no_extension_out,
            );
            */
        }
    };
    (
        ($first:ident, $second:ident, $third:ident),
        $crates_io_out:expr,
        $http_repo_out:expr,
        $http_repo_no_extension_out:expr,
        $ssh_repo_out:expr,
        $ssh_repo_no_extension_out:expr,
    ) => {
        paste! {
            test_module_extended!(
                [<$first _ $second _and_ $third>],
                ($first, $second, $third),
                $crates_io_out,
                $http_repo_out,
                $http_repo_no_extension_out,
                $ssh_repo_out,
                $ssh_repo_no_extension_out,
            );
            /*
            test_module_extended!(
                [<$second _ $first _and_ $third>],
                ($second, $first, $third),
                $crates_io_out,
                $http_repo_out,
                $http_repo_no_extension_out,
                $ssh_repo_out,
                $ssh_repo_no_extension_out,
            );
            test_module_extended!(
                [<$third _ $first _and_ $second>],
                ($third, $first, $second),
                $crates_io_out,
                $http_repo_out,
                $http_repo_no_extension_out,
                $ssh_repo_out,
                $ssh_repo_no_extension_out,
            );
            test_module_extended!(
                [<$first _ $third _and_ $second>],
                ($first, $third, $second),
                $crates_io_out,
                $http_repo_out,
                $http_repo_no_extension_out,
                $ssh_repo_out,
                $ssh_repo_no_extension_out,
            );
            test_module_extended!(
                [<$second _ $third _and_ $first>],
                ($second, $third, $first),
                $crates_io_out,
                $http_repo_out,
                $http_repo_no_extension_out,
                $ssh_repo_out,
                $ssh_repo_no_extension_out,
            );
            test_module_extended!(
                [<$third _ $second _and_ $first>],
                ($third, $second, $first),
                $crates_io_out,
                $http_repo_out,
                $http_repo_no_extension_out,
                $ssh_repo_out,
                $ssh_repo_no_extension_out,
            );
            */
        }
    };
    (
        ($first:ident, $second:ident, $third:ident, $fourth:ident),
        $http_repo_out:expr,
        $http_repo_no_extension_out:expr,
        $ssh_repo_out:expr,
        $ssh_repo_no_extension_out:expr,
    ) => {
        paste! {
            test_module_extended!(
                [<$first _ $second _ $third _and_ $fourth>],
                ($first, $second, $third, $fourth),
                $http_repo_out,
                $http_repo_no_extension_out,
                $ssh_repo_out,
                $ssh_repo_no_extension_out,
            );
            /*
            test_module_extended!(
                [<$first _ $second _ $fourth _and_ $third>],
                ($first, $second, $fourth, $third),
                $http_repo_out,
                $http_repo_no_extension_out,
                $ssh_repo_out,
                $ssh_repo_no_extension_out,
            );
            test_module_extended!(
                [<$first _ $third _ $second _and_ $fourth>],
                ($first, $third, $second, $fourth),
                $http_repo_out,
                $http_repo_no_extension_out,
                $ssh_repo_out,
                $ssh_repo_no_extension_out,
            );
            test_module_extended!(
                [<$first _ $third _ $fourth _and_ $second>],
                ($first, $third, $fourth, $second),
                $http_repo_out,
                $http_repo_no_extension_out,
                $ssh_repo_out,
                $ssh_repo_no_extension_out,
            );
            test_module_extended!(
                [<$first _ $fourth _ $second _and_ $third>],
                ($first, $fourth, $second, $third),
                $http_repo_out,
                $http_repo_no_extension_out,
                $ssh_repo_out,
                $ssh_repo_no_extension_out,
            );
            test_module_extended!(
                [<$first _ $fourth _ $third _and_ $second>],
                ($first, $fourth, $third, $second),
                $http_repo_out,
                $http_repo_no_extension_out,
                $ssh_repo_out,
                $ssh_repo_no_extension_out,
            );
            test_module_extended!(
                [<$second _ $first _ $third _and_ $fourth>],
                ($second, $first, $third, $fourth),
                $http_repo_out,
                $http_repo_no_extension_out,
                $ssh_repo_out,
                $ssh_repo_no_extension_out,
            );
            test_module_extended!(
                [<$second _ $first _ $fourth _and_ $third>],
                ($second, $first, $fourth, $third),
                $http_repo_out,
                $http_repo_no_extension_out,
                $ssh_repo_out,
                $ssh_repo_no_extension_out,
            );
            test_module_extended!(
                [<$second _ $third _ $first _and_ $fourth>],
                ($second, $third, $first, $fourth),
                $http_repo_out,
                $http_repo_no_extension_out,
                $ssh_repo_out,
                $ssh_repo_no_extension_out,
            );
            test_module_extended!(
                [<$second _ $third _ $fourth _and_ $first>],
                ($second, $third, $fourth, $first),
                $http_repo_out,
                $http_repo_no_extension_out,
                $ssh_repo_out,
                $ssh_repo_no_extension_out,
            );
            test_module_extended!(
                [<$second _ $fourth _ $first _and_ $third>],
                ($second, $fourth, $first, $third),
                $http_repo_out,
                $http_repo_no_extension_out,
                $ssh_repo_out,
                $ssh_repo_no_extension_out,
            );
            test_module_extended!(
                [<$second _ $fourth _ $third _and_ $first>],
                ($second, $fourth, $third, $first),
                $http_repo_out,
                $http_repo_no_extension_out,
                $ssh_repo_out,
                $ssh_repo_no_extension_out,
            );
            test_module_extended!(
                [<$third _ $first _ $second _and_ $fourth>],
                ($third, $first, $second, $fourth),
                $http_repo_out,
                $http_repo_no_extension_out,
                $ssh_repo_out,
                $ssh_repo_no_extension_out,
            );
            test_module_extended!(
                [<$third _ $first _ $fourth _and_ $second>],
                ($third, $first, $fourth, $second),
                $http_repo_out,
                $http_repo_no_extension_out,
                $ssh_repo_out,
                $ssh_repo_no_extension_out,
            );
            test_module_extended!(
                [<$third _ $second _ $first _and_ $fourth>],
                ($third, $second, $first, $fourth),
                $http_repo_out,
                $http_repo_no_extension_out,
                $ssh_repo_out,
                $ssh_repo_no_extension_out,
            );
            test_module_extended!(
                [<$third _ $second _ $fourth _and_ $first>],
                ($third, $second, $fourth, $first),
                $http_repo_out,
                $http_repo_no_extension_out,
                $ssh_repo_out,
                $ssh_repo_no_extension_out,
            );
            test_module_extended!(
                [<$third _ $fourth _ $first _and_ $second>],
                ($third, $fourth, $first, $second),
                $http_repo_out,
                $http_repo_no_extension_out,
                $ssh_repo_out,
                $ssh_repo_no_extension_out,
            );
            test_module_extended!(
                [<$third _ $fourth _ $second _and_ $first>],
                ($third, $fourth, $second, $first),
                $http_repo_out,
                $http_repo_no_extension_out,
                $ssh_repo_out,
                $ssh_repo_no_extension_out,
            );
            test_module_extended!(
                [<$fourth _ $first _ $second _and_ $third>],
                ($fourth, $first, $second, $third),
                $http_repo_out,
                $http_repo_no_extension_out,
                $ssh_repo_out,
                $ssh_repo_no_extension_out,
            );
            test_module_extended!(
                [<$fourth _ $first _ $third _and_ $second>],
                ($fourth, $first, $third, $second),
                $http_repo_out,
                $http_repo_no_extension_out,
                $ssh_repo_out,
                $ssh_repo_no_extension_out,
            );
            test_module_extended!(
                [<$fourth _ $second _ $first _and_ $third>],
                ($fourth, $second, $first, $third),
                $http_repo_out,
                $http_repo_no_extension_out,
                $ssh_repo_out,
                $ssh_repo_no_extension_out,
            );
            test_module_extended!(
                [<$fourth _ $second _ $third _and_ $first>],
                ($fourth, $second, $third, $first),
                $http_repo_out,
                $http_repo_no_extension_out,
                $ssh_repo_out,
                $ssh_repo_no_extension_out,
            );
            test_module_extended!(
                [<$fourth _ $third _ $first _and_ $second>],
                ($fourth, $third, $first, $second),
                $http_repo_out,
                $http_repo_no_extension_out,
                $ssh_repo_out,
                $ssh_repo_no_extension_out,
            );
            test_module_extended!(
                [<$fourth _ $third _ $second _and_ $first>],
                ($fourth, $third, $second, $first),
                $http_repo_out,
                $http_repo_no_extension_out,
                $ssh_repo_out,
                $ssh_repo_no_extension_out,
            );
            */
        }
    };
}

struct Inputs(String, Dependency);

impl Inputs {
    fn crates_io() -> Self {
        let name = "anyhow";
        Self(
            name.to_string(),
            Dependency::CratesIo {
                name: name.to_string(),
                version: None,
                features: Vec::new(),
                default_features: true,
            },
        )
    }

    fn http_repo() -> Self {
        let url = "https://github.com/tokio-rs/tokio.git";
        Self(
            url.to_string(),
            Dependency::Repository {
                name: "tokio".to_string(),
                version: None,
                features: Vec::new(),
                default_features: true,
                url: url.to_string(),
                branch: None,
                rev: None,
            },
        )
    }

    fn http_repo_no_extension() -> Self {
        let url = "https://github.com/clap-rs/clap";
        Self(
            url.to_string(),
            Dependency::Repository {
                name: "clap".to_string(),
                version: None,
                features: Vec::new(),
                default_features: true,
                url: url.to_string(),
                branch: None,
                rev: None,
            },
        )
    }

    fn ssh_repo() -> Self {
        let url = "ssh://git@github.com/rust-random/rand.git";
        Self(
            url.to_string(),
            Dependency::Repository {
                name: "rand".to_string(),
                version: None,
                features: Vec::new(),
                default_features: true,
                url: url.to_string(),
                branch: None,
                rev: None,
            },
        )
    }

    fn ssh_repo_no_extension() -> Self {
        let url = "ssh://git@github.com/rust-lang/log";
        Self(
            url.to_string(),
            Dependency::Repository {
                name: "log".to_string(),
                version: None,
                features: Vec::new(),
                default_features: true,
                url: url.to_string(),
                branch: None,
                rev: None,
            },
        )
    }
}

test_module!(
    "anyhow = \"*\"",
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\" }",
    "clap = { git = \"https://github.com/clap-rs/clap\" }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\" }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\" }",
);

test_module!(
    version,
    |mut inputs: Inputs| {
        match &mut inputs.1 {
            Dependency::CratesIo { version, .. } => {
                let v = "1.0.100";
                inputs.0.push('=');
                inputs.0.push_str(v);
                *version = Some(v.to_string());
            }
            Dependency::Repository { name, version, .. } => {
                let v = match name.as_ref() {
                    "tokio" => "1.48",
                    "clap" => "4.5.50",
                    "rand" => "0.9",
                    _ => "0.4.28",
                };
                inputs.0.push('=');
                inputs.0.push_str(v);
                *version = Some(v.to_string());
            }
        }
        inputs
    },
    "anyhow = \"1.0.100\"",
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", version = \"1.48\" }",
    "clap = { git = \"https://github.com/clap-rs/clap\", version = \"4.5.50\" }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", version = \"0.9\" }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", version = \"0.4.28\" }",
);

test_module!(
    exact_version,
    |mut inputs: Inputs| {
        match &mut inputs.1 {
            Dependency::CratesIo { version, .. } => {
                let v = "=1.0.100";
                inputs.0.push('=');
                inputs.0.push_str(v);
                *version = Some(v.to_string());
            }
            Dependency::Repository { name, version, .. } => {
                let v = match name.as_ref() {
                    "tokio" => "=1.48",
                    "clap" => "=4.5.50",
                    "rand" => "=0.9",
                    _ => "=0.4.28",
                };
                inputs.0.push('=');
                inputs.0.push_str(v);
                *version = Some(v.to_string());
            }
        }
        inputs
    },
    "anyhow = \"=1.0.100\"",
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", version = \"=1.48\" }",
    "clap = { git = \"https://github.com/clap-rs/clap\", version = \"=4.5.50\" }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", version = \"=0.9\" }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", version = \"=0.4.28\" }",
);

test_module!(
    maximal_version,
    |mut inputs: Inputs| {
        match &mut inputs.1 {
            Dependency::CratesIo { version, .. } => {
                let v = "<1.0.100";
                inputs.0.push('=');
                inputs.0.push_str(v);
                *version = Some(v.to_string());
            }
            Dependency::Repository { name, version, .. } => {
                let v = match name.as_ref() {
                    "tokio" => "<1.48",
                    "clap" => "<4.5.50",
                    "rand" => "<0.9",
                    _ => "<0.4.28",
                };
                inputs.0.push('=');
                inputs.0.push_str(v);
                *version = Some(v.to_string());
            }
        }
        inputs
    },
    "anyhow = \"<1.0.100\"",
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", version = \"<1.48\" }",
    "clap = { git = \"https://github.com/clap-rs/clap\", version = \"<4.5.50\" }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", version = \"<0.9\" }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", version = \"<0.4.28\" }",
);

test_module!(
    maximal_or_equal_version,
    |mut inputs: Inputs| {
        match &mut inputs.1 {
            Dependency::CratesIo { version, .. } => {
                let v = "<=1.0.100";
                inputs.0.push('=');
                inputs.0.push_str(v);
                *version = Some(v.to_string());
            }
            Dependency::Repository { name, version, .. } => {
                let v = match name.as_ref() {
                    "tokio" => "<=1.48",
                    "clap" => "<=4.5.50",
                    "rand" => "<=0.9",
                    _ => "<=0.4.28",
                };
                inputs.0.push('=');
                inputs.0.push_str(v);
                *version = Some(v.to_string());
            }
        }
        inputs
    },
    "anyhow = \"<=1.0.100\"",
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", version = \"<=1.48\" }",
    "clap = { git = \"https://github.com/clap-rs/clap\", version = \"<=4.5.50\" }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", version = \"<=0.9\" }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", version = \"<=0.4.28\" }",
);

test_module!(
    minimal_version,
    |mut inputs: Inputs| {
        match &mut inputs.1 {
            Dependency::CratesIo { version, .. } => {
                let v = ">1.0.100";
                inputs.0.push('=');
                inputs.0.push_str(v);
                *version = Some(v.to_string());
            }
            Dependency::Repository { name, version, .. } => {
                let v = match name.as_ref() {
                    "tokio" => ">1.48",
                    "clap" => ">4.5.50",
                    "rand" => ">0.9",
                    _ => ">0.4.28",
                };
                inputs.0.push('=');
                inputs.0.push_str(v);
                *version = Some(v.to_string());
            }
        }
        inputs
    },
    "anyhow = \">1.0.100\"",
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", version = \">1.48\" }",
    "clap = { git = \"https://github.com/clap-rs/clap\", version = \">4.5.50\" }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", version = \">0.9\" }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", version = \">0.4.28\" }",
);

test_module!(
    minimal_or_equal_version,
    |mut inputs: Inputs| {
        match &mut inputs.1 {
            Dependency::CratesIo { version, .. } => {
                let v = ">=1.0.100";
                inputs.0.push('=');
                inputs.0.push_str(v);
                *version = Some(v.to_string());
            }
            Dependency::Repository { name, version, .. } => {
                let v = match name.as_ref() {
                    "tokio" => ">=1.48",
                    "clap" => ">=4.5.50",
                    "rand" => ">=0.9",
                    _ => ">=0.4.28",
                };
                inputs.0.push('=');
                inputs.0.push_str(v);
                *version = Some(v.to_string());
            }
        }
        inputs
    },
    "anyhow = \">=1.0.100\"",
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", version = \">=1.48\" }",
    "clap = { git = \"https://github.com/clap-rs/clap\", version = \">=4.5.50\" }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", version = \">=0.9\" }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", version = \">=0.4.28\" }",
);

test_module!(
    major_version,
    |mut inputs: Inputs| {
        match &mut inputs.1 {
            Dependency::CratesIo { version, .. } => {
                let v = "~1.0.100";
                inputs.0.push('=');
                inputs.0.push_str(v);
                *version = Some(v.to_string());
            }
            Dependency::Repository { name, version, .. } => {
                let v = match name.as_ref() {
                    "tokio" => "~1.48",
                    "clap" => "~4.5.50",
                    "rand" => "~0.9",
                    _ => "~0.4.28",
                };
                inputs.0.push('=');
                inputs.0.push_str(v);
                *version = Some(v.to_string());
            }
        }
        inputs
    },
    "anyhow = \"~1.0.100\"",
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", version = \"~1.48\" }",
    "clap = { git = \"https://github.com/clap-rs/clap\", version = \"~4.5.50\" }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", version = \"~0.9\" }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", version = \"~0.4.28\" }",
);

test_module!(
    feature,
    |mut inputs: Inputs| {
        match &mut inputs.1 {
            Dependency::CratesIo { features, .. } => {
                let f = "backtrace";
                inputs.0.push('+');
                inputs.0.push_str(f);
                *features = vec![f.to_string()];
            }
            Dependency::Repository { name, features, .. } => {
                let f = match name.as_ref() {
                    "tokio" => "io_std",
                    "clap" => "derive",
                    "rand" => "small_rng",
                    _ => "kv_std",
                };
                inputs.0.push('+');
                inputs.0.push_str(f);
                *features = vec![f.to_string()];
            }
        }
        inputs
    },
    "anyhow = { version = \"*\", features = [\"backtrace\"] }",
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", features = [\"io_std\"] }",
    "clap = { git = \"https://github.com/clap-rs/clap\", features = [\"derive\"] }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", features = [\"small_rng\"] }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", features = [\"kv_std\"] }",
);

test_module!(
    features,
    |mut inputs: Inputs| {
        match &mut inputs.1 {
            Dependency::CratesIo { features, .. } => {
                let f = ["backtrace", "std"];
                *features = f
                    .iter()
                    .map(|x| {
                        inputs.0.push('+');
                        inputs.0.push_str(x);
                        x.to_string()
                    })
                    .collect();
            }
            Dependency::Repository { name, features, .. } => {
                let f = match name.as_ref() {
                    "tokio" => ["io_std", "io_utils"],
                    "clap" => ["derive", "cargo"],
                    "rand" => ["small_rng", "os_rng"],
                    _ => ["kv_std", "kv_sval"],
                };
                *features = f
                    .iter()
                    .map(|x| {
                        inputs.0.push('+');
                        inputs.0.push_str(x);
                        x.to_string()
                    })
                    .collect();
            }
        }
        inputs
    },
    "anyhow = { version = \"*\", features = [\"backtrace\", \"std\"] }",
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", features = [\"io_std\", \"io_utils\"] }",
    "clap = { git = \"https://github.com/clap-rs/clap\", features = [\"derive\", \"cargo\"] }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", features = [\"small_rng\", \"os_rng\"] }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", features = [\"kv_std\", \"kv_sval\"] }",
);

test_module!(
    no_default_features,
    |mut inputs: Inputs| {
        inputs.0.push_str("!default");
        match &mut inputs.1 {
            Dependency::CratesIo {
                default_features, ..
            }
            | Dependency::Repository {
                default_features, ..
            } => {
                *default_features = false;
            }
        }
        inputs
    },
    "anyhow = { version = \"*\", default-features = false }",
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", default-features = false }",
    "clap = { git = \"https://github.com/clap-rs/clap\", default-features = false }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", default-features = false }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", default-features = false }",
);

test_module!(
    branch,
    |mut inputs: Inputs| {
        match &mut inputs.1 {
            Dependency::CratesIo { .. } => {}
            Dependency::Repository { name, branch, .. } => {
                let b = match name.as_ref() {
                    "tokio" => "compat",
                    "clap" => "modular",
                    "rand" => "thread_rng",
                    _ => "0.3.x",
                };
                inputs.0.push('#');
                inputs.0.push_str(b);
                *branch = Some(b.to_string());
            }
        }
        inputs
    },
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", branch = \"compat\" }",
    "clap = { git = \"https://github.com/clap-rs/clap\", branch = \"modular\" }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", branch = \"thread_rng\" }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", branch = \"0.3.x\" }",
);

test_module!(
    rev,
    |mut inputs: Inputs| {
        match &mut inputs.1 {
            Dependency::CratesIo { .. } => {}
            Dependency::Repository { name, rev, .. } => {
                let r = match name.as_ref() {
                    "tokio" => "556820f",
                    "clap" => "c7c761f988684ad97c8b2c521b05cf7f8192b3eb",
                    "rand" => "db993ec",
                    _ => "6e1735597bb21c5d979a077395df85e1d633e077",
                };
                inputs.0.push('#');
                inputs.0.push_str(r);
                *rev = Some(r.to_string());
            }
        }
        inputs
    },
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", rev = \"556820f\" }",
    "clap = { git = \"https://github.com/clap-rs/clap\", rev = \"c7c761f988684ad97c8b2c521b05cf7f8192b3eb\" }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", rev = \"db993ec\" }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", rev = \"6e1735597bb21c5d979a077395df85e1d633e077\" }",
);

test_modules!(
    (version, feature),
    "anyhow = { version = \"1.0.100\", features = [\"backtrace\"] }",
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", version = \"1.48\", features = [\"io_std\"] }",
    "clap = { git = \"https://github.com/clap-rs/clap\", version = \"4.5.50\", features = [\"derive\"] }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", version = \"0.9\", features = [\"small_rng\"] }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", version = \"0.4.28\", features = [\"kv_std\"] }",
);

test_modules!(
    (version, features),
    "anyhow = { version = \"1.0.100\", features = [\"backtrace\", \"std\"] }",
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", version = \"1.48\", features = [\"io_std\", \"io_utils\"] }",
    "clap = { git = \"https://github.com/clap-rs/clap\", version = \"4.5.50\", features = [\"derive\", \"cargo\"] }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", version = \"0.9\", features = [\"small_rng\", \"os_rng\"] }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", version = \"0.4.28\", features = [\"kv_std\", \"kv_sval\"] }",
);

test_modules!(
    (exact_version, feature),
    "anyhow = { version = \"=1.0.100\", features = [\"backtrace\"] }",
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", version = \"=1.48\", features = [\"io_std\"] }",
    "clap = { git = \"https://github.com/clap-rs/clap\", version = \"=4.5.50\", features = [\"derive\"] }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", version = \"=0.9\", features = [\"small_rng\"] }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", version = \"=0.4.28\", features = [\"kv_std\"] }",
);

test_modules!(
    (exact_version, features),
    "anyhow = { version = \"=1.0.100\", features = [\"backtrace\", \"std\"] }",
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", version = \"=1.48\", features = [\"io_std\", \"io_utils\"] }",
    "clap = { git = \"https://github.com/clap-rs/clap\", version = \"=4.5.50\", features = [\"derive\", \"cargo\"] }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", version = \"=0.9\", features = [\"small_rng\", \"os_rng\"] }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", version = \"=0.4.28\", features = [\"kv_std\", \"kv_sval\"] }",
);

test_modules!(
    (maximal_version, feature),
    "anyhow = { version = \"<1.0.100\", features = [\"backtrace\"] }",
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", version = \"<1.48\", features = [\"io_std\"] }",
    "clap = { git = \"https://github.com/clap-rs/clap\", version = \"<4.5.50\", features = [\"derive\"] }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", version = \"<0.9\", features = [\"small_rng\"] }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", version = \"<0.4.28\", features = [\"kv_std\"] }",
);

test_modules!(
    (maximal_or_equal_version, feature),
    "anyhow = { version = \"<=1.0.100\", features = [\"backtrace\"] }",
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", version = \"<=1.48\", features = [\"io_std\"] }",
    "clap = { git = \"https://github.com/clap-rs/clap\", version = \"<=4.5.50\", features = [\"derive\"] }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", version = \"<=0.9\", features = [\"small_rng\"] }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", version = \"<=0.4.28\", features = [\"kv_std\"] }",
);

test_modules!(
    (minimal_version, feature),
    "anyhow = { version = \">1.0.100\", features = [\"backtrace\"] }",
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", version = \">1.48\", features = [\"io_std\"] }",
    "clap = { git = \"https://github.com/clap-rs/clap\", version = \">4.5.50\", features = [\"derive\"] }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", version = \">0.9\", features = [\"small_rng\"] }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", version = \">0.4.28\", features = [\"kv_std\"] }",
);

test_modules!(
    (minimal_or_equal_version, feature),
    "anyhow = { version = \">=1.0.100\", features = [\"backtrace\"] }",
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", version = \">=1.48\", features = [\"io_std\"] }",
    "clap = { git = \"https://github.com/clap-rs/clap\", version = \">=4.5.50\", features = [\"derive\"] }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", version = \">=0.9\", features = [\"small_rng\"] }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", version = \">=0.4.28\", features = [\"kv_std\"] }",
);

test_modules!(
    (major_version, feature),
    "anyhow = { version = \"~1.0.100\", features = [\"backtrace\"] }",
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", version = \"~1.48\", features = [\"io_std\"] }",
    "clap = { git = \"https://github.com/clap-rs/clap\", version = \"~4.5.50\", features = [\"derive\"] }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", version = \"~0.9\", features = [\"small_rng\"] }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", version = \"~0.4.28\", features = [\"kv_std\"] }",
);

test_modules!(
    (maximal_version, features),
    "anyhow = { version = \"<1.0.100\", features = [\"backtrace\", \"std\"] }",
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", version = \"<1.48\", features = [\"io_std\", \"io_utils\"] }",
    "clap = { git = \"https://github.com/clap-rs/clap\", version = \"<4.5.50\", features = [\"derive\", \"cargo\"] }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", version = \"<0.9\", features = [\"small_rng\", \"os_rng\"] }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", version = \"<0.4.28\", features = [\"kv_std\", \"kv_sval\"] }",
);

test_modules!(
    (maximal_or_equal_version, features),
    "anyhow = { version = \"<=1.0.100\", features = [\"backtrace\", \"std\"] }",
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", version = \"<=1.48\", features = [\"io_std\", \"io_utils\"] }",
    "clap = { git = \"https://github.com/clap-rs/clap\", version = \"<=4.5.50\", features = [\"derive\", \"cargo\"] }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", version = \"<=0.9\", features = [\"small_rng\", \"os_rng\"] }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", version = \"<=0.4.28\", features = [\"kv_std\", \"kv_sval\"] }",
);

test_modules!(
    (minimal_version, features),
    "anyhow = { version = \">1.0.100\", features = [\"backtrace\", \"std\"] }",
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", version = \">1.48\", features = [\"io_std\", \"io_utils\"] }",
    "clap = { git = \"https://github.com/clap-rs/clap\", version = \">4.5.50\", features = [\"derive\", \"cargo\"] }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", version = \">0.9\", features = [\"small_rng\", \"os_rng\"] }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", version = \">0.4.28\", features = [\"kv_std\", \"kv_sval\"] }",
);

test_modules!(
    (minimal_or_equal_version, features),
    "anyhow = { version = \">=1.0.100\", features = [\"backtrace\", \"std\"] }",
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", version = \">=1.48\", features = [\"io_std\", \"io_utils\"] }",
    "clap = { git = \"https://github.com/clap-rs/clap\", version = \">=4.5.50\", features = [\"derive\", \"cargo\"] }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", version = \">=0.9\", features = [\"small_rng\", \"os_rng\"] }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", version = \">=0.4.28\", features = [\"kv_std\", \"kv_sval\"] }",
);

test_modules!(
    (major_version, features),
    "anyhow = { version = \"~1.0.100\", features = [\"backtrace\", \"std\"] }",
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", version = \"~1.48\", features = [\"io_std\", \"io_utils\"] }",
    "clap = { git = \"https://github.com/clap-rs/clap\", version = \"~4.5.50\", features = [\"derive\", \"cargo\"] }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", version = \"~0.9\", features = [\"small_rng\", \"os_rng\"] }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", version = \"~0.4.28\", features = [\"kv_std\", \"kv_sval\"] }",
);

test_modules!(
    (version, no_default_features),
    "anyhow = { version = \"1.0.100\", default-features = false }",
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", version = \"1.48\", default-features = false }",
    "clap = { git = \"https://github.com/clap-rs/clap\", version = \"4.5.50\", default-features = false }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", version = \"0.9\", default-features = false }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", version = \"0.4.28\", default-features = false }",
);

test_modules!(
    (exact_version, no_default_features),
    "anyhow = { version = \"=1.0.100\", default-features = false }",
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", version = \"=1.48\", default-features = false }",
    "clap = { git = \"https://github.com/clap-rs/clap\", version = \"=4.5.50\", default-features = false }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", version = \"=0.9\", default-features = false }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", version = \"=0.4.28\", default-features = false }",
);

test_modules!(
    (maximal_version, no_default_features),
    "anyhow = { version = \"<1.0.100\", default-features = false }",
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", version = \"<1.48\", default-features = false }",
    "clap = { git = \"https://github.com/clap-rs/clap\", version = \"<4.5.50\", default-features = false }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", version = \"<0.9\", default-features = false }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", version = \"<0.4.28\", default-features = false }",
);

test_modules!(
    (maximal_or_equal_version, no_default_features),
    "anyhow = { version = \"<=1.0.100\", default-features = false }",
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", version = \"<=1.48\", default-features = false }",
    "clap = { git = \"https://github.com/clap-rs/clap\", version = \"<=4.5.50\", default-features = false }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", version = \"<=0.9\", default-features = false }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", version = \"<=0.4.28\", default-features = false }",
);

test_modules!(
    (minimal_version, no_default_features),
    "anyhow = { version = \">1.0.100\", default-features = false }",
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", version = \">1.48\", default-features = false }",
    "clap = { git = \"https://github.com/clap-rs/clap\", version = \">4.5.50\", default-features = false }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", version = \">0.9\", default-features = false }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", version = \">0.4.28\", default-features = false }",
);

test_modules!(
    (minimal_or_equal_version, no_default_features),
    "anyhow = { version = \">=1.0.100\", default-features = false }",
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", version = \">=1.48\", default-features = false }",
    "clap = { git = \"https://github.com/clap-rs/clap\", version = \">=4.5.50\", default-features = false }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", version = \">=0.9\", default-features = false }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", version = \">=0.4.28\", default-features = false }",
);

test_modules!(
    (major_version, no_default_features),
    "anyhow = { version = \"~1.0.100\", default-features = false }",
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", version = \"~1.48\", default-features = false }",
    "clap = { git = \"https://github.com/clap-rs/clap\", version = \"~4.5.50\", default-features = false }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", version = \"~0.9\", default-features = false }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", version = \"~0.4.28\", default-features = false }",
);

test_modules!(
    (branch, version),
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", branch = \"compat\", version = \"1.48\" }",
    "clap = { git = \"https://github.com/clap-rs/clap\", branch = \"modular\", version = \"4.5.50\" }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", branch = \"thread_rng\", version = \"0.9\" }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", branch = \"0.3.x\", version = \"0.4.28\" }",
);

test_modules!(
    (rev, version),
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", rev = \"556820f\", version = \"1.48\" }",
    "clap = { git = \"https://github.com/clap-rs/clap\", rev = \"c7c761f988684ad97c8b2c521b05cf7f8192b3eb\", version = \"4.5.50\" }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", rev = \"db993ec\", version = \"0.9\" }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", rev = \"6e1735597bb21c5d979a077395df85e1d633e077\", version = \"0.4.28\" }",
);

test_modules!(
    (branch, exact_version),
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", branch = \"compat\", version = \"=1.48\" }",
    "clap = { git = \"https://github.com/clap-rs/clap\", branch = \"modular\", version = \"=4.5.50\" }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", branch = \"thread_rng\", version = \"=0.9\" }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", branch = \"0.3.x\", version = \"=0.4.28\" }",
);

test_modules!(
    (rev, exact_version),
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", rev = \"556820f\", version = \"=1.48\" }",
    "clap = { git = \"https://github.com/clap-rs/clap\", rev = \"c7c761f988684ad97c8b2c521b05cf7f8192b3eb\", version = \"=4.5.50\" }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", rev = \"db993ec\", version = \"=0.9\" }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", rev = \"6e1735597bb21c5d979a077395df85e1d633e077\", version = \"=0.4.28\" }",
);

test_modules!(
    (branch, maximal_version),
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", branch = \"compat\", version = \"<1.48\" }",
    "clap = { git = \"https://github.com/clap-rs/clap\", branch = \"modular\", version = \"<4.5.50\" }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", branch = \"thread_rng\", version = \"<0.9\" }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", branch = \"0.3.x\", version = \"<0.4.28\" }",
);

test_modules!(
    (branch, maximal_or_equal_version),
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", branch = \"compat\", version = \"<=1.48\" }",
    "clap = { git = \"https://github.com/clap-rs/clap\", branch = \"modular\", version = \"<=4.5.50\" }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", branch = \"thread_rng\", version = \"<=0.9\" }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", branch = \"0.3.x\", version = \"<=0.4.28\" }",
);

test_modules!(
    (branch, minimal_version),
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", branch = \"compat\", version = \">1.48\" }",
    "clap = { git = \"https://github.com/clap-rs/clap\", branch = \"modular\", version = \">4.5.50\" }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", branch = \"thread_rng\", version = \">0.9\" }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", branch = \"0.3.x\", version = \">0.4.28\" }",
);

test_modules!(
    (branch, minimal_or_equal_version),
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", branch = \"compat\", version = \">=1.48\" }",
    "clap = { git = \"https://github.com/clap-rs/clap\", branch = \"modular\", version = \">=4.5.50\" }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", branch = \"thread_rng\", version = \">=0.9\" }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", branch = \"0.3.x\", version = \">=0.4.28\" }",
);

test_modules!(
    (branch, major_version),
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", branch = \"compat\", version = \"~1.48\" }",
    "clap = { git = \"https://github.com/clap-rs/clap\", branch = \"modular\", version = \"~4.5.50\" }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", branch = \"thread_rng\", version = \"~0.9\" }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", branch = \"0.3.x\", version = \"~0.4.28\" }",
);

test_modules!(
    (rev, maximal_version),
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", rev = \"556820f\", version = \"<1.48\" }",
    "clap = { git = \"https://github.com/clap-rs/clap\", rev = \"c7c761f988684ad97c8b2c521b05cf7f8192b3eb\", version = \"<4.5.50\" }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", rev = \"db993ec\", version = \"<0.9\" }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", rev = \"6e1735597bb21c5d979a077395df85e1d633e077\", version = \"<0.4.28\" }",
);

test_modules!(
    (rev, maximal_or_equal_version),
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", rev = \"556820f\", version = \"<=1.48\" }",
    "clap = { git = \"https://github.com/clap-rs/clap\", rev = \"c7c761f988684ad97c8b2c521b05cf7f8192b3eb\", version = \"<=4.5.50\" }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", rev = \"db993ec\", version = \"<=0.9\" }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", rev = \"6e1735597bb21c5d979a077395df85e1d633e077\", version = \"<=0.4.28\" }",
);

test_modules!(
    (rev, minimal_version),
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", rev = \"556820f\", version = \">1.48\" }",
    "clap = { git = \"https://github.com/clap-rs/clap\", rev = \"c7c761f988684ad97c8b2c521b05cf7f8192b3eb\", version = \">4.5.50\" }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", rev = \"db993ec\", version = \">0.9\" }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", rev = \"6e1735597bb21c5d979a077395df85e1d633e077\", version = \">0.4.28\" }",
);

test_modules!(
    (rev, minimal_or_equal_version),
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", rev = \"556820f\", version = \">=1.48\" }",
    "clap = { git = \"https://github.com/clap-rs/clap\", rev = \"c7c761f988684ad97c8b2c521b05cf7f8192b3eb\", version = \">=4.5.50\" }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", rev = \"db993ec\", version = \">=0.9\" }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", rev = \"6e1735597bb21c5d979a077395df85e1d633e077\", version = \">=0.4.28\" }",
);

test_modules!(
    (rev, major_version),
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", rev = \"556820f\", version = \"~1.48\" }",
    "clap = { git = \"https://github.com/clap-rs/clap\", rev = \"c7c761f988684ad97c8b2c521b05cf7f8192b3eb\", version = \"~4.5.50\" }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", rev = \"db993ec\", version = \"~0.9\" }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", rev = \"6e1735597bb21c5d979a077395df85e1d633e077\", version = \"~0.4.28\" }",
);

test_modules!(
    (branch, feature),
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", branch = \"compat\", features = [\"io_std\"] }",
    "clap = { git = \"https://github.com/clap-rs/clap\", branch = \"modular\", features = [\"derive\"] }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", branch = \"thread_rng\", features = [\"small_rng\"] }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", branch = \"0.3.x\", features = [\"kv_std\"] }",
);

test_modules!(
    (rev, feature),
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", rev = \"556820f\", features = [\"io_std\"] }",
    "clap = { git = \"https://github.com/clap-rs/clap\", rev = \"c7c761f988684ad97c8b2c521b05cf7f8192b3eb\", features = [\"derive\"] }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", rev = \"db993ec\", features = [\"small_rng\"] }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", rev = \"6e1735597bb21c5d979a077395df85e1d633e077\", features = [\"kv_std\"] }",
);

test_modules!(
    (branch, features),
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", branch = \"compat\", features = [\"io_std\", \"io_utils\"] }",
    "clap = { git = \"https://github.com/clap-rs/clap\", branch = \"modular\", features = [\"derive\", \"cargo\"] }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", branch = \"thread_rng\", features = [\"small_rng\", \"os_rng\"] }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", branch = \"0.3.x\", features = [\"kv_std\", \"kv_sval\"] }",
);

test_modules!(
    (rev, features),
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", rev = \"556820f\", features = [\"io_std\", \"io_utils\"] }",
    "clap = { git = \"https://github.com/clap-rs/clap\", rev = \"c7c761f988684ad97c8b2c521b05cf7f8192b3eb\", features = [\"derive\", \"cargo\"] }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", rev = \"db993ec\", features = [\"small_rng\", \"os_rng\"] }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", rev = \"6e1735597bb21c5d979a077395df85e1d633e077\", features = [\"kv_std\", \"kv_sval\"] }",
);

test_modules!(
    (no_default_features, feature),
    "anyhow = { version = \"*\", default-features = false, features = [\"backtrace\"] }",
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", default-features = false, features = [\"io_std\"] }",
    "clap = { git = \"https://github.com/clap-rs/clap\", default-features = false, features = [\"derive\"] }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", default-features = false, features = [\"small_rng\"] }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", default-features = false, features = [\"kv_std\"] }",
);

test_modules!(
    (no_default_features, features),
    "anyhow = { version = \"*\", default-features = false, features = [\"backtrace\", \"std\"] }",
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", default-features = false, features = [\"io_std\", \"io_utils\"] }",
    "clap = { git = \"https://github.com/clap-rs/clap\", default-features = false, features = [\"derive\", \"cargo\"] }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", default-features = false, features = [\"small_rng\", \"os_rng\"] }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", default-features = false, features = [\"kv_std\", \"kv_sval\"] }",
);

test_modules!(
    (branch, no_default_features),
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", branch = \"compat\", default-features = false }",
    "clap = { git = \"https://github.com/clap-rs/clap\", branch = \"modular\", default-features = false }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", branch = \"thread_rng\", default-features = false }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", branch = \"0.3.x\", default-features = false }",
);

test_modules!(
    (rev, no_default_features),
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", rev = \"556820f\", default-features = false }",
    "clap = { git = \"https://github.com/clap-rs/clap\", rev = \"c7c761f988684ad97c8b2c521b05cf7f8192b3eb\", default-features = false }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", rev = \"db993ec\", default-features = false }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", rev = \"6e1735597bb21c5d979a077395df85e1d633e077\", default-features = false }",
);

test_modules!(
    (branch, version, feature),
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", branch = \"compat\", version = \"1.48\", features = [\"io_std\"] }",
    "clap = { git = \"https://github.com/clap-rs/clap\", branch = \"modular\", version = \"4.5.50\", features = [\"derive\"] }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", branch = \"thread_rng\", version = \"0.9\", features = [\"small_rng\"] }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", branch = \"0.3.x\", version = \"0.4.28\", features = [\"kv_std\"] }",
);

test_modules!(
    (rev, version, feature),
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", rev = \"556820f\", version = \"1.48\", features = [\"io_std\"] }",
    "clap = { git = \"https://github.com/clap-rs/clap\", rev = \"c7c761f988684ad97c8b2c521b05cf7f8192b3eb\", version = \"4.5.50\", features = [\"derive\"] }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", rev = \"db993ec\", version = \"0.9\", features = [\"small_rng\"] }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", rev = \"6e1735597bb21c5d979a077395df85e1d633e077\", version = \"0.4.28\", features = [\"kv_std\"] }",
);

test_modules!(
    (branch, version, features),
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", branch = \"compat\", version = \"1.48\", features = [\"io_std\", \"io_utils\"] }",
    "clap = { git = \"https://github.com/clap-rs/clap\", branch = \"modular\", version = \"4.5.50\", features = [\"derive\", \"cargo\"] }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", branch = \"thread_rng\", version = \"0.9\", features = [\"small_rng\", \"os_rng\"] }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", branch = \"0.3.x\", version = \"0.4.28\", features = [\"kv_std\", \"kv_sval\"] }",
);

test_modules!(
    (rev, version, features),
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", rev = \"556820f\", version = \"1.48\", features = [\"io_std\", \"io_utils\"] }",
    "clap = { git = \"https://github.com/clap-rs/clap\", rev = \"c7c761f988684ad97c8b2c521b05cf7f8192b3eb\", version = \"4.5.50\", features = [\"derive\", \"cargo\"] }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", rev = \"db993ec\", version = \"0.9\", features = [\"small_rng\", \"os_rng\"] }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", rev = \"6e1735597bb21c5d979a077395df85e1d633e077\", version = \"0.4.28\", features = [\"kv_std\", \"kv_sval\"] }",
);

test_modules!(
    (branch, exact_version, feature),
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", branch = \"compat\", version = \"=1.48\", features = [\"io_std\"] }",
    "clap = { git = \"https://github.com/clap-rs/clap\", branch = \"modular\", version = \"=4.5.50\", features = [\"derive\"] }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", branch = \"thread_rng\", version = \"=0.9\", features = [\"small_rng\"] }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", branch = \"0.3.x\", version = \"=0.4.28\", features = [\"kv_std\"] }",
);

test_modules!(
    (rev, exact_version, feature),
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", rev = \"556820f\", version = \"=1.48\", features = [\"io_std\"] }",
    "clap = { git = \"https://github.com/clap-rs/clap\", rev = \"c7c761f988684ad97c8b2c521b05cf7f8192b3eb\", version = \"=4.5.50\", features = [\"derive\"] }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", rev = \"db993ec\", version = \"=0.9\", features = [\"small_rng\"] }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", rev = \"6e1735597bb21c5d979a077395df85e1d633e077\", version = \"=0.4.28\", features = [\"kv_std\"] }",
);

test_modules!(
    (branch, exact_version, features),
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", branch = \"compat\", version = \"=1.48\", features = [\"io_std\", \"io_utils\"] }",
    "clap = { git = \"https://github.com/clap-rs/clap\", branch = \"modular\", version = \"=4.5.50\", features = [\"derive\", \"cargo\"] }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", branch = \"thread_rng\", version = \"=0.9\", features = [\"small_rng\", \"os_rng\"] }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", branch = \"0.3.x\", version = \"=0.4.28\", features = [\"kv_std\", \"kv_sval\"] }",
);

test_modules!(
    (rev, exact_version, features),
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", rev = \"556820f\", version = \"=1.48\", features = [\"io_std\", \"io_utils\"] }",
    "clap = { git = \"https://github.com/clap-rs/clap\", rev = \"c7c761f988684ad97c8b2c521b05cf7f8192b3eb\", version = \"=4.5.50\", features = [\"derive\", \"cargo\"] }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", rev = \"db993ec\", version = \"=0.9\", features = [\"small_rng\", \"os_rng\"] }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", rev = \"6e1735597bb21c5d979a077395df85e1d633e077\", version = \"=0.4.28\", features = [\"kv_std\", \"kv_sval\"] }",
);

test_modules!(
    (branch, maximal_version, feature),
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", branch = \"compat\", version = \"<1.48\", features = [\"io_std\"] }",
    "clap = { git = \"https://github.com/clap-rs/clap\", branch = \"modular\", version = \"<4.5.50\", features = [\"derive\"] }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", branch = \"thread_rng\", version = \"<0.9\", features = [\"small_rng\"] }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", branch = \"0.3.x\", version = \"<0.4.28\", features = [\"kv_std\"] }",
);

test_modules!(
    (branch, maximal_or_equal_version, feature),
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", branch = \"compat\", version = \"<=1.48\", features = [\"io_std\"] }",
    "clap = { git = \"https://github.com/clap-rs/clap\", branch = \"modular\", version = \"<=4.5.50\", features = [\"derive\"] }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", branch = \"thread_rng\", version = \"<=0.9\", features = [\"small_rng\"] }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", branch = \"0.3.x\", version = \"<=0.4.28\", features = [\"kv_std\"] }",
);

test_modules!(
    (branch, minimal_version, feature),
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", branch = \"compat\", version = \">1.48\", features = [\"io_std\"] }",
    "clap = { git = \"https://github.com/clap-rs/clap\", branch = \"modular\", version = \">4.5.50\", features = [\"derive\"] }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", branch = \"thread_rng\", version = \">0.9\", features = [\"small_rng\"] }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", branch = \"0.3.x\", version = \">0.4.28\", features = [\"kv_std\"] }",
);

test_modules!(
    (branch, minimal_or_equal_version, feature),
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", branch = \"compat\", version = \">=1.48\", features = [\"io_std\"] }",
    "clap = { git = \"https://github.com/clap-rs/clap\", branch = \"modular\", version = \">=4.5.50\", features = [\"derive\"] }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", branch = \"thread_rng\", version = \">=0.9\", features = [\"small_rng\"] }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", branch = \"0.3.x\", version = \">=0.4.28\", features = [\"kv_std\"] }",
);

test_modules!(
    (branch, major_version, feature),
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", branch = \"compat\", version = \"~1.48\", features = [\"io_std\"] }",
    "clap = { git = \"https://github.com/clap-rs/clap\", branch = \"modular\", version = \"~4.5.50\", features = [\"derive\"] }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", branch = \"thread_rng\", version = \"~0.9\", features = [\"small_rng\"] }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", branch = \"0.3.x\", version = \"~0.4.28\", features = [\"kv_std\"] }",
);

test_modules!(
    (rev, maximal_version, feature),
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", rev = \"556820f\", version = \"<1.48\", features = [\"io_std\"] }",
    "clap = { git = \"https://github.com/clap-rs/clap\", rev = \"c7c761f988684ad97c8b2c521b05cf7f8192b3eb\", version = \"<4.5.50\", features = [\"derive\"] }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", rev = \"db993ec\", version = \"<0.9\", features = [\"small_rng\"] }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", rev = \"6e1735597bb21c5d979a077395df85e1d633e077\", version = \"<0.4.28\", features = [\"kv_std\"] }",
);

test_modules!(
    (rev, maximal_or_equal_version, feature),
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", rev = \"556820f\", version = \"<=1.48\", features = [\"io_std\"] }",
    "clap = { git = \"https://github.com/clap-rs/clap\", rev = \"c7c761f988684ad97c8b2c521b05cf7f8192b3eb\", version = \"<=4.5.50\", features = [\"derive\"] }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", rev = \"db993ec\", version = \"<=0.9\", features = [\"small_rng\"] }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", rev = \"6e1735597bb21c5d979a077395df85e1d633e077\", version = \"<=0.4.28\", features = [\"kv_std\"] }",
);

test_modules!(
    (rev, minimal_version, feature),
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", rev = \"556820f\", version = \">1.48\", features = [\"io_std\"] }",
    "clap = { git = \"https://github.com/clap-rs/clap\", rev = \"c7c761f988684ad97c8b2c521b05cf7f8192b3eb\", version = \">4.5.50\", features = [\"derive\"] }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", rev = \"db993ec\", version = \">0.9\", features = [\"small_rng\"] }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", rev = \"6e1735597bb21c5d979a077395df85e1d633e077\", version = \">0.4.28\", features = [\"kv_std\"] }",
);

test_modules!(
    (rev, minimal_or_equal_version, feature),
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", rev = \"556820f\", version = \">=1.48\", features = [\"io_std\"] }",
    "clap = { git = \"https://github.com/clap-rs/clap\", rev = \"c7c761f988684ad97c8b2c521b05cf7f8192b3eb\", version = \">=4.5.50\", features = [\"derive\"] }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", rev = \"db993ec\", version = \">=0.9\", features = [\"small_rng\"] }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", rev = \"6e1735597bb21c5d979a077395df85e1d633e077\", version = \">=0.4.28\", features = [\"kv_std\"] }",
);

test_modules!(
    (rev, major_version, feature),
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", rev = \"556820f\", version = \"~1.48\", features = [\"io_std\"] }",
    "clap = { git = \"https://github.com/clap-rs/clap\", rev = \"c7c761f988684ad97c8b2c521b05cf7f8192b3eb\", version = \"~4.5.50\", features = [\"derive\"] }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", rev = \"db993ec\", version = \"~0.9\", features = [\"small_rng\"] }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", rev = \"6e1735597bb21c5d979a077395df85e1d633e077\", version = \"~0.4.28\", features = [\"kv_std\"] }",
);

test_modules!(
    (branch, maximal_version, features),
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", branch = \"compat\", version = \"<1.48\", features = [\"io_std\", \"io_utils\"] }",
    "clap = { git = \"https://github.com/clap-rs/clap\", branch = \"modular\", version = \"<4.5.50\", features = [\"derive\", \"cargo\"] }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", branch = \"thread_rng\", version = \"<0.9\", features = [\"small_rng\", \"os_rng\"] }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", branch = \"0.3.x\", version = \"<0.4.28\", features = [\"kv_std\", \"kv_sval\"] }",
);

test_modules!(
    (branch, maximal_or_equal_version, features),
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", branch = \"compat\", version = \"<=1.48\", features = [\"io_std\", \"io_utils\"] }",
    "clap = { git = \"https://github.com/clap-rs/clap\", branch = \"modular\", version = \"<=4.5.50\", features = [\"derive\", \"cargo\"] }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", branch = \"thread_rng\", version = \"<=0.9\", features = [\"small_rng\", \"os_rng\"] }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", branch = \"0.3.x\", version = \"<=0.4.28\", features = [\"kv_std\", \"kv_sval\"] }",
);

test_modules!(
    (branch, minimal_version, features),
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", branch = \"compat\", version = \">1.48\", features = [\"io_std\", \"io_utils\"] }",
    "clap = { git = \"https://github.com/clap-rs/clap\", branch = \"modular\", version = \">4.5.50\", features = [\"derive\", \"cargo\"] }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", branch = \"thread_rng\", version = \">0.9\", features = [\"small_rng\", \"os_rng\"] }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", branch = \"0.3.x\", version = \">0.4.28\", features = [\"kv_std\", \"kv_sval\"] }",
);

test_modules!(
    (branch, minimal_or_equal_version, features),
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", branch = \"compat\", version = \">=1.48\", features = [\"io_std\", \"io_utils\"] }",
    "clap = { git = \"https://github.com/clap-rs/clap\", branch = \"modular\", version = \">=4.5.50\", features = [\"derive\", \"cargo\"] }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", branch = \"thread_rng\", version = \">=0.9\", features = [\"small_rng\", \"os_rng\"] }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", branch = \"0.3.x\", version = \">=0.4.28\", features = [\"kv_std\", \"kv_sval\"] }",
);

test_modules!(
    (branch, major_version, features),
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", branch = \"compat\", version = \"~1.48\", features = [\"io_std\", \"io_utils\"] }",
    "clap = { git = \"https://github.com/clap-rs/clap\", branch = \"modular\", version = \"~4.5.50\", features = [\"derive\", \"cargo\"] }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", branch = \"thread_rng\", version = \"~0.9\", features = [\"small_rng\", \"os_rng\"] }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", branch = \"0.3.x\", version = \"~0.4.28\", features = [\"kv_std\", \"kv_sval\"] }",
);

test_modules!(
    (rev, maximal_version, features),
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", rev = \"556820f\", version = \"<1.48\", features = [\"io_std\", \"io_utils\"] }",
    "clap = { git = \"https://github.com/clap-rs/clap\", rev = \"c7c761f988684ad97c8b2c521b05cf7f8192b3eb\", version = \"<4.5.50\", features = [\"derive\", \"cargo\"] }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", rev = \"db993ec\", version = \"<0.9\", features = [\"small_rng\", \"os_rng\"] }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", rev = \"6e1735597bb21c5d979a077395df85e1d633e077\", version = \"<0.4.28\", features = [\"kv_std\", \"kv_sval\"] }",
);

test_modules!(
    (rev, maximal_or_equal_version, features),
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", rev = \"556820f\", version = \"<=1.48\", features = [\"io_std\", \"io_utils\"] }",
    "clap = { git = \"https://github.com/clap-rs/clap\", rev = \"c7c761f988684ad97c8b2c521b05cf7f8192b3eb\", version = \"<=4.5.50\", features = [\"derive\", \"cargo\"] }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", rev = \"db993ec\", version = \"<=0.9\", features = [\"small_rng\", \"os_rng\"] }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", rev = \"6e1735597bb21c5d979a077395df85e1d633e077\", version = \"<=0.4.28\", features = [\"kv_std\", \"kv_sval\"] }",
);

test_modules!(
    (rev, minimal_version, features),
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", rev = \"556820f\", version = \">1.48\", features = [\"io_std\", \"io_utils\"] }",
    "clap = { git = \"https://github.com/clap-rs/clap\", rev = \"c7c761f988684ad97c8b2c521b05cf7f8192b3eb\", version = \">4.5.50\", features = [\"derive\", \"cargo\"] }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", rev = \"db993ec\", version = \">0.9\", features = [\"small_rng\", \"os_rng\"] }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", rev = \"6e1735597bb21c5d979a077395df85e1d633e077\", version = \">0.4.28\", features = [\"kv_std\", \"kv_sval\"] }",
);

test_modules!(
    (rev, minimal_or_equal_version, features),
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", rev = \"556820f\", version = \">=1.48\", features = [\"io_std\", \"io_utils\"] }",
    "clap = { git = \"https://github.com/clap-rs/clap\", rev = \"c7c761f988684ad97c8b2c521b05cf7f8192b3eb\", version = \">=4.5.50\", features = [\"derive\", \"cargo\"] }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", rev = \"db993ec\", version = \">=0.9\", features = [\"small_rng\", \"os_rng\"] }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", rev = \"6e1735597bb21c5d979a077395df85e1d633e077\", version = \">=0.4.28\", features = [\"kv_std\", \"kv_sval\"] }",
);

test_modules!(
    (rev, major_version, features),
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", rev = \"556820f\", version = \"~1.48\", features = [\"io_std\", \"io_utils\"] }",
    "clap = { git = \"https://github.com/clap-rs/clap\", rev = \"c7c761f988684ad97c8b2c521b05cf7f8192b3eb\", version = \"~4.5.50\", features = [\"derive\", \"cargo\"] }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", rev = \"db993ec\", version = \"~0.9\", features = [\"small_rng\", \"os_rng\"] }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", rev = \"6e1735597bb21c5d979a077395df85e1d633e077\", version = \"~0.4.28\", features = [\"kv_std\", \"kv_sval\"] }",
);

test_modules!(
    (version, no_default_features, feature),
    "anyhow = { version = \"1.0.100\", default-features = false, features = [\"backtrace\"] }",
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", version = \"1.48\", default-features = false, features = [\"io_std\"] }",
    "clap = { git = \"https://github.com/clap-rs/clap\", version = \"4.5.50\", default-features = false, features = [\"derive\"] }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", version = \"0.9\", default-features = false, features = [\"small_rng\"] }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", version = \"0.4.28\", default-features = false, features = [\"kv_std\"] }",
);

test_modules!(
    (version, no_default_features, features),
    "anyhow = { version = \"1.0.100\", default-features = false, features = [\"backtrace\", \"std\"] }",
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", version = \"1.48\", default-features = false, features = [\"io_std\", \"io_utils\"] }",
    "clap = { git = \"https://github.com/clap-rs/clap\", version = \"4.5.50\", default-features = false, features = [\"derive\", \"cargo\"] }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", version = \"0.9\", default-features = false, features = [\"small_rng\", \"os_rng\"] }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", version = \"0.4.28\", default-features = false, features = [\"kv_std\", \"kv_sval\"] }",
);

test_modules!(
    (exact_version, no_default_features, feature),
    "anyhow = { version = \"=1.0.100\", default-features = false, features = [\"backtrace\"] }",
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", version = \"=1.48\", default-features = false, features = [\"io_std\"] }",
    "clap = { git = \"https://github.com/clap-rs/clap\", version = \"=4.5.50\", default-features = false, features = [\"derive\"] }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", version = \"=0.9\", default-features = false, features = [\"small_rng\"] }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", version = \"=0.4.28\", default-features = false, features = [\"kv_std\"] }",
);

test_modules!(
    (exact_version, no_default_features, features),
    "anyhow = { version = \"=1.0.100\", default-features = false, features = [\"backtrace\", \"std\"] }",
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", version = \"=1.48\", default-features = false, features = [\"io_std\", \"io_utils\"] }",
    "clap = { git = \"https://github.com/clap-rs/clap\", version = \"=4.5.50\", default-features = false, features = [\"derive\", \"cargo\"] }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", version = \"=0.9\", default-features = false, features = [\"small_rng\", \"os_rng\"] }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", version = \"=0.4.28\", default-features = false, features = [\"kv_std\", \"kv_sval\"] }",
);

test_modules!(
    (maximal_version, no_default_features, feature),
    "anyhow = { version = \"<1.0.100\", default-features = false, features = [\"backtrace\"] }",
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", version = \"<1.48\", default-features = false, features = [\"io_std\"] }",
    "clap = { git = \"https://github.com/clap-rs/clap\", version = \"<4.5.50\", default-features = false, features = [\"derive\"] }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", version = \"<0.9\", default-features = false, features = [\"small_rng\"] }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", version = \"<0.4.28\", default-features = false, features = [\"kv_std\"] }",
);

test_modules!(
    (maximal_or_equal_version, no_default_features, feature),
    "anyhow = { version = \"<=1.0.100\", default-features = false, features = [\"backtrace\"] }",
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", version = \"<=1.48\", default-features = false, features = [\"io_std\"] }",
    "clap = { git = \"https://github.com/clap-rs/clap\", version = \"<=4.5.50\", default-features = false, features = [\"derive\"] }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", version = \"<=0.9\", default-features = false, features = [\"small_rng\"] }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", version = \"<=0.4.28\", default-features = false, features = [\"kv_std\"] }",
);

test_modules!(
    (minimal_version, no_default_features, feature),
    "anyhow = { version = \">1.0.100\", default-features = false, features = [\"backtrace\"] }",
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", version = \">1.48\", default-features = false, features = [\"io_std\"] }",
    "clap = { git = \"https://github.com/clap-rs/clap\", version = \">4.5.50\", default-features = false, features = [\"derive\"] }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", version = \">0.9\", default-features = false, features = [\"small_rng\"] }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", version = \">0.4.28\", default-features = false, features = [\"kv_std\"] }",
);

test_modules!(
    (minimal_or_equal_version, no_default_features, feature),
    "anyhow = { version = \">=1.0.100\", default-features = false, features = [\"backtrace\"] }",
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", version = \">=1.48\", default-features = false, features = [\"io_std\"] }",
    "clap = { git = \"https://github.com/clap-rs/clap\", version = \">=4.5.50\", default-features = false, features = [\"derive\"] }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", version = \">=0.9\", default-features = false, features = [\"small_rng\"] }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", version = \">=0.4.28\", default-features = false, features = [\"kv_std\"] }",
);

test_modules!(
    (major_version, no_default_features, feature),
    "anyhow = { version = \"~1.0.100\", default-features = false, features = [\"backtrace\"] }",
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", version = \"~1.48\", default-features = false, features = [\"io_std\"] }",
    "clap = { git = \"https://github.com/clap-rs/clap\", version = \"~4.5.50\", default-features = false, features = [\"derive\"] }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", version = \"~0.9\", default-features = false, features = [\"small_rng\"] }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", version = \"~0.4.28\", default-features = false, features = [\"kv_std\"] }",
);

test_modules!(
    (maximal_version, no_default_features, features),
    "anyhow = { version = \"<1.0.100\", default-features = false, features = [\"backtrace\", \"std\"] }",
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", version = \"<1.48\", default-features = false, features = [\"io_std\", \"io_utils\"] }",
    "clap = { git = \"https://github.com/clap-rs/clap\", version = \"<4.5.50\", default-features = false, features = [\"derive\", \"cargo\"] }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", version = \"<0.9\", default-features = false, features = [\"small_rng\", \"os_rng\"] }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", version = \"<0.4.28\", default-features = false, features = [\"kv_std\", \"kv_sval\"] }",
);

test_modules!(
    (maximal_or_equal_version, no_default_features, features),
    "anyhow = { version = \"<=1.0.100\", default-features = false, features = [\"backtrace\", \"std\"] }",
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", version = \"<=1.48\", default-features = false, features = [\"io_std\", \"io_utils\"] }",
    "clap = { git = \"https://github.com/clap-rs/clap\", version = \"<=4.5.50\", default-features = false, features = [\"derive\", \"cargo\"] }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", version = \"<=0.9\", default-features = false, features = [\"small_rng\", \"os_rng\"] }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", version = \"<=0.4.28\", default-features = false, features = [\"kv_std\", \"kv_sval\"] }",
);

test_modules!(
    (minimal_version, no_default_features, features),
    "anyhow = { version = \">1.0.100\", default-features = false, features = [\"backtrace\", \"std\"] }",
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", version = \">1.48\", default-features = false, features = [\"io_std\", \"io_utils\"] }",
    "clap = { git = \"https://github.com/clap-rs/clap\", version = \">4.5.50\", default-features = false, features = [\"derive\", \"cargo\"] }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", version = \">0.9\", default-features = false, features = [\"small_rng\", \"os_rng\"] }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", version = \">0.4.28\", default-features = false, features = [\"kv_std\", \"kv_sval\"] }",
);

test_modules!(
    (minimal_or_equal_version, no_default_features, features),
    "anyhow = { version = \">=1.0.100\", default-features = false, features = [\"backtrace\", \"std\"] }",
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", version = \">=1.48\", default-features = false, features = [\"io_std\", \"io_utils\"] }",
    "clap = { git = \"https://github.com/clap-rs/clap\", version = \">=4.5.50\", default-features = false, features = [\"derive\", \"cargo\"] }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", version = \">=0.9\", default-features = false, features = [\"small_rng\", \"os_rng\"] }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", version = \">=0.4.28\", default-features = false, features = [\"kv_std\", \"kv_sval\"] }",
);

test_modules!(
    (major_version, no_default_features, features),
    "anyhow = { version = \"~1.0.100\", default-features = false, features = [\"backtrace\", \"std\"] }",
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", version = \"~1.48\", default-features = false, features = [\"io_std\", \"io_utils\"] }",
    "clap = { git = \"https://github.com/clap-rs/clap\", version = \"~4.5.50\", default-features = false, features = [\"derive\", \"cargo\"] }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", version = \"~0.9\", default-features = false, features = [\"small_rng\", \"os_rng\"] }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", version = \"~0.4.28\", default-features = false, features = [\"kv_std\", \"kv_sval\"] }",
);

test_modules!(
    (branch, no_default_features, feature),
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", branch = \"compat\", default-features = false, features = [\"io_std\"] }",
    "clap = { git = \"https://github.com/clap-rs/clap\", branch = \"modular\", default-features = false, features = [\"derive\"] }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", branch = \"thread_rng\", default-features = false, features = [\"small_rng\"] }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", branch = \"0.3.x\", default-features = false, features = [\"kv_std\"] }",
);

test_modules!(
    (rev, no_default_features, feature),
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", rev = \"556820f\", default-features = false, features = [\"io_std\"] }",
    "clap = { git = \"https://github.com/clap-rs/clap\", rev = \"c7c761f988684ad97c8b2c521b05cf7f8192b3eb\", default-features = false, features = [\"derive\"] }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", rev = \"db993ec\", default-features = false, features = [\"small_rng\"] }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", rev = \"6e1735597bb21c5d979a077395df85e1d633e077\", default-features = false, features = [\"kv_std\"] }",
);

test_modules!(
    (branch, no_default_features, features),
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", branch = \"compat\", default-features = false, features = [\"io_std\", \"io_utils\"] }",
    "clap = { git = \"https://github.com/clap-rs/clap\", branch = \"modular\", default-features = false, features = [\"derive\", \"cargo\"] }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", branch = \"thread_rng\", default-features = false, features = [\"small_rng\", \"os_rng\"] }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", branch = \"0.3.x\", default-features = false, features = [\"kv_std\", \"kv_sval\"] }",
);

test_modules!(
    (rev, no_default_features, features),
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", rev = \"556820f\", default-features = false, features = [\"io_std\", \"io_utils\"] }",
    "clap = { git = \"https://github.com/clap-rs/clap\", rev = \"c7c761f988684ad97c8b2c521b05cf7f8192b3eb\", default-features = false, features = [\"derive\", \"cargo\"] }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", rev = \"db993ec\", default-features = false, features = [\"small_rng\", \"os_rng\"] }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", rev = \"6e1735597bb21c5d979a077395df85e1d633e077\", default-features = false, features = [\"kv_std\", \"kv_sval\"] }",
);

test_modules!(
    (branch, version, no_default_features, feature),
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", branch = \"compat\", version = \"1.48\", default-features = false, features = [\"io_std\"] }",
    "clap = { git = \"https://github.com/clap-rs/clap\", branch = \"modular\", version = \"4.5.50\", default-features = false, features = [\"derive\"] }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", branch = \"thread_rng\", version = \"0.9\", default-features = false, features = [\"small_rng\"] }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", branch = \"0.3.x\", version = \"0.4.28\", default-features = false, features = [\"kv_std\"] }",
);

test_modules!(
    (rev, version, no_default_features, feature),
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", rev = \"556820f\", version = \"1.48\", default-features = false, features = [\"io_std\"] }",
    "clap = { git = \"https://github.com/clap-rs/clap\", rev = \"c7c761f988684ad97c8b2c521b05cf7f8192b3eb\", version = \"4.5.50\", default-features = false, features = [\"derive\"] }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", rev = \"db993ec\", version = \"0.9\", default-features = false, features = [\"small_rng\"] }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", rev = \"6e1735597bb21c5d979a077395df85e1d633e077\", version = \"0.4.28\", default-features = false, features = [\"kv_std\"] }",
);

test_modules!(
    (branch, version, no_default_features, features),
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", branch = \"compat\", version = \"1.48\", default-features = false, features = [\"io_std\", \"io_utils\"] }",
    "clap = { git = \"https://github.com/clap-rs/clap\", branch = \"modular\", version = \"4.5.50\", default-features = false, features = [\"derive\", \"cargo\"] }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", branch = \"thread_rng\", version = \"0.9\", default-features = false, features = [\"small_rng\", \"os_rng\"] }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", branch = \"0.3.x\", version = \"0.4.28\", default-features = false, features = [\"kv_std\", \"kv_sval\"] }",
);

test_modules!(
    (rev, version, no_default_features, features),
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", rev = \"556820f\", version = \"1.48\", default-features = false, features = [\"io_std\", \"io_utils\"] }",
    "clap = { git = \"https://github.com/clap-rs/clap\", rev = \"c7c761f988684ad97c8b2c521b05cf7f8192b3eb\", version = \"4.5.50\", default-features = false, features = [\"derive\", \"cargo\"] }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", rev = \"db993ec\", version = \"0.9\", default-features = false, features = [\"small_rng\", \"os_rng\"] }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", rev = \"6e1735597bb21c5d979a077395df85e1d633e077\", version = \"0.4.28\", default-features = false, features = [\"kv_std\", \"kv_sval\"] }",
);

test_modules!(
    (branch, exact_version, no_default_features, feature),
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", branch = \"compat\", version = \"=1.48\", default-features = false, features = [\"io_std\"] }",
    "clap = { git = \"https://github.com/clap-rs/clap\", branch = \"modular\", version = \"=4.5.50\", default-features = false, features = [\"derive\"] }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", branch = \"thread_rng\", version = \"=0.9\", default-features = false, features = [\"small_rng\"] }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", branch = \"0.3.x\", version = \"=0.4.28\", default-features = false, features = [\"kv_std\"] }",
);

test_modules!(
    (rev, exact_version, no_default_features, feature),
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", rev = \"556820f\", version = \"=1.48\", default-features = false, features = [\"io_std\"] }",
    "clap = { git = \"https://github.com/clap-rs/clap\", rev = \"c7c761f988684ad97c8b2c521b05cf7f8192b3eb\", version = \"=4.5.50\", default-features = false, features = [\"derive\"] }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", rev = \"db993ec\", version = \"=0.9\", default-features = false, features = [\"small_rng\"] }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", rev = \"6e1735597bb21c5d979a077395df85e1d633e077\", version = \"=0.4.28\", default-features = false, features = [\"kv_std\"] }",
);

test_modules!(
    (branch, exact_version, no_default_features, features),
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", branch = \"compat\", version = \"=1.48\", default-features = false, features = [\"io_std\", \"io_utils\"] }",
    "clap = { git = \"https://github.com/clap-rs/clap\", branch = \"modular\", version = \"=4.5.50\", default-features = false, features = [\"derive\", \"cargo\"] }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", branch = \"thread_rng\", version = \"=0.9\", default-features = false, features = [\"small_rng\", \"os_rng\"] }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", branch = \"0.3.x\", version = \"=0.4.28\", default-features = false, features = [\"kv_std\", \"kv_sval\"] }",
);

test_modules!(
    (rev, exact_version, no_default_features, features),
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", rev = \"556820f\", version = \"=1.48\", default-features = false, features = [\"io_std\", \"io_utils\"] }",
    "clap = { git = \"https://github.com/clap-rs/clap\", rev = \"c7c761f988684ad97c8b2c521b05cf7f8192b3eb\", version = \"=4.5.50\", default-features = false, features = [\"derive\", \"cargo\"] }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", rev = \"db993ec\", version = \"=0.9\", default-features = false, features = [\"small_rng\", \"os_rng\"] }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", rev = \"6e1735597bb21c5d979a077395df85e1d633e077\", version = \"=0.4.28\", default-features = false, features = [\"kv_std\", \"kv_sval\"] }",
);

test_modules!(
    (branch, maximal_version, no_default_features, feature),
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", branch = \"compat\", version = \"<1.48\", default-features = false, features = [\"io_std\"] }",
    "clap = { git = \"https://github.com/clap-rs/clap\", branch = \"modular\", version = \"<4.5.50\", default-features = false, features = [\"derive\"] }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", branch = \"thread_rng\", version = \"<0.9\", default-features = false, features = [\"small_rng\"] }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", branch = \"0.3.x\", version = \"<0.4.28\", default-features = false, features = [\"kv_std\"] }",
);

test_modules!(
    (
        branch,
        maximal_or_equal_version,
        no_default_features,
        feature
    ),
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", branch = \"compat\", version = \"<=1.48\", default-features = false, features = [\"io_std\"] }",
    "clap = { git = \"https://github.com/clap-rs/clap\", branch = \"modular\", version = \"<=4.5.50\", default-features = false, features = [\"derive\"] }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", branch = \"thread_rng\", version = \"<=0.9\", default-features = false, features = [\"small_rng\"] }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", branch = \"0.3.x\", version = \"<=0.4.28\", default-features = false, features = [\"kv_std\"] }",
);

test_modules!(
    (branch, minimal_version, no_default_features, feature),
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", branch = \"compat\", version = \">1.48\", default-features = false, features = [\"io_std\"] }",
    "clap = { git = \"https://github.com/clap-rs/clap\", branch = \"modular\", version = \">4.5.50\", default-features = false, features = [\"derive\"] }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", branch = \"thread_rng\", version = \">0.9\", default-features = false, features = [\"small_rng\"] }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", branch = \"0.3.x\", version = \">0.4.28\", default-features = false, features = [\"kv_std\"] }",
);

test_modules!(
    (
        branch,
        minimal_or_equal_version,
        no_default_features,
        feature
    ),
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", branch = \"compat\", version = \">=1.48\", default-features = false, features = [\"io_std\"] }",
    "clap = { git = \"https://github.com/clap-rs/clap\", branch = \"modular\", version = \">=4.5.50\", default-features = false, features = [\"derive\"] }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", branch = \"thread_rng\", version = \">=0.9\", default-features = false, features = [\"small_rng\"] }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", branch = \"0.3.x\", version = \">=0.4.28\", default-features = false, features = [\"kv_std\"] }",
);

test_modules!(
    (branch, major_version, no_default_features, feature),
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", branch = \"compat\", version = \"~1.48\", default-features = false, features = [\"io_std\"] }",
    "clap = { git = \"https://github.com/clap-rs/clap\", branch = \"modular\", version = \"~4.5.50\", default-features = false, features = [\"derive\"] }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", branch = \"thread_rng\", version = \"~0.9\", default-features = false, features = [\"small_rng\"] }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", branch = \"0.3.x\", version = \"~0.4.28\", default-features = false, features = [\"kv_std\"] }",
);

test_modules!(
    (rev, maximal_version, no_default_features, feature),
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", rev = \"556820f\", version = \"<1.48\", default-features = false, features = [\"io_std\"] }",
    "clap = { git = \"https://github.com/clap-rs/clap\", rev = \"c7c761f988684ad97c8b2c521b05cf7f8192b3eb\", version = \"<4.5.50\", default-features = false, features = [\"derive\"] }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", rev = \"db993ec\", version = \"<0.9\", default-features = false, features = [\"small_rng\"] }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", rev = \"6e1735597bb21c5d979a077395df85e1d633e077\", version = \"<0.4.28\", default-features = false, features = [\"kv_std\"] }",
);

test_modules!(
    (rev, maximal_or_equal_version, no_default_features, feature),
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", rev = \"556820f\", version = \"<=1.48\", default-features = false, features = [\"io_std\"] }",
    "clap = { git = \"https://github.com/clap-rs/clap\", rev = \"c7c761f988684ad97c8b2c521b05cf7f8192b3eb\", version = \"<=4.5.50\", default-features = false, features = [\"derive\"] }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", rev = \"db993ec\", version = \"<=0.9\", default-features = false, features = [\"small_rng\"] }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", rev = \"6e1735597bb21c5d979a077395df85e1d633e077\", version = \"<=0.4.28\", default-features = false, features = [\"kv_std\"] }",
);

test_modules!(
    (rev, minimal_version, no_default_features, feature),
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", rev = \"556820f\", version = \">1.48\", default-features = false, features = [\"io_std\"] }",
    "clap = { git = \"https://github.com/clap-rs/clap\", rev = \"c7c761f988684ad97c8b2c521b05cf7f8192b3eb\", version = \">4.5.50\", default-features = false, features = [\"derive\"] }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", rev = \"db993ec\", version = \">0.9\", default-features = false, features = [\"small_rng\"] }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", rev = \"6e1735597bb21c5d979a077395df85e1d633e077\", version = \">0.4.28\", default-features = false, features = [\"kv_std\"] }",
);

test_modules!(
    (rev, minimal_or_equal_version, no_default_features, feature),
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", rev = \"556820f\", version = \">=1.48\", default-features = false, features = [\"io_std\"] }",
    "clap = { git = \"https://github.com/clap-rs/clap\", rev = \"c7c761f988684ad97c8b2c521b05cf7f8192b3eb\", version = \">=4.5.50\", default-features = false, features = [\"derive\"] }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", rev = \"db993ec\", version = \">=0.9\", default-features = false, features = [\"small_rng\"] }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", rev = \"6e1735597bb21c5d979a077395df85e1d633e077\", version = \">=0.4.28\", default-features = false, features = [\"kv_std\"] }",
);

test_modules!(
    (rev, major_version, no_default_features, feature),
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", rev = \"556820f\", version = \"~1.48\", default-features = false, features = [\"io_std\"] }",
    "clap = { git = \"https://github.com/clap-rs/clap\", rev = \"c7c761f988684ad97c8b2c521b05cf7f8192b3eb\", version = \"~4.5.50\", default-features = false, features = [\"derive\"] }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", rev = \"db993ec\", version = \"~0.9\", default-features = false, features = [\"small_rng\"] }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", rev = \"6e1735597bb21c5d979a077395df85e1d633e077\", version = \"~0.4.28\", default-features = false, features = [\"kv_std\"] }",
);

test_modules!(
    (branch, maximal_version, no_default_features, features),
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", branch = \"compat\", version = \"<1.48\", default-features = false, features = [\"io_std\", \"io_utils\"] }",
    "clap = { git = \"https://github.com/clap-rs/clap\", branch = \"modular\", version = \"<4.5.50\", default-features = false, features = [\"derive\", \"cargo\"] }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", branch = \"thread_rng\", version = \"<0.9\", default-features = false, features = [\"small_rng\", \"os_rng\"] }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", branch = \"0.3.x\", version = \"<0.4.28\", default-features = false, features = [\"kv_std\", \"kv_sval\"] }",
);

test_modules!(
    (
        branch,
        maximal_or_equal_version,
        no_default_features,
        features
    ),
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", branch = \"compat\", version = \"<=1.48\", default-features = false, features = [\"io_std\", \"io_utils\"] }",
    "clap = { git = \"https://github.com/clap-rs/clap\", branch = \"modular\", version = \"<=4.5.50\", default-features = false, features = [\"derive\", \"cargo\"] }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", branch = \"thread_rng\", version = \"<=0.9\", default-features = false, features = [\"small_rng\", \"os_rng\"] }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", branch = \"0.3.x\", version = \"<=0.4.28\", default-features = false, features = [\"kv_std\", \"kv_sval\"] }",
);

test_modules!(
    (branch, minimal_version, no_default_features, features),
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", branch = \"compat\", version = \">1.48\", default-features = false, features = [\"io_std\", \"io_utils\"] }",
    "clap = { git = \"https://github.com/clap-rs/clap\", branch = \"modular\", version = \">4.5.50\", default-features = false, features = [\"derive\", \"cargo\"] }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", branch = \"thread_rng\", version = \">0.9\", default-features = false, features = [\"small_rng\", \"os_rng\"] }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", branch = \"0.3.x\", version = \">0.4.28\", default-features = false, features = [\"kv_std\", \"kv_sval\"] }",
);

test_modules!(
    (
        branch,
        minimal_or_equal_version,
        no_default_features,
        features
    ),
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", branch = \"compat\", version = \">=1.48\", default-features = false, features = [\"io_std\", \"io_utils\"] }",
    "clap = { git = \"https://github.com/clap-rs/clap\", branch = \"modular\", version = \">=4.5.50\", default-features = false, features = [\"derive\", \"cargo\"] }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", branch = \"thread_rng\", version = \">=0.9\", default-features = false, features = [\"small_rng\", \"os_rng\"] }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", branch = \"0.3.x\", version = \">=0.4.28\", default-features = false, features = [\"kv_std\", \"kv_sval\"] }",
);

test_modules!(
    (branch, major_version, no_default_features, features),
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", branch = \"compat\", version = \"~1.48\", default-features = false, features = [\"io_std\", \"io_utils\"] }",
    "clap = { git = \"https://github.com/clap-rs/clap\", branch = \"modular\", version = \"~4.5.50\", default-features = false, features = [\"derive\", \"cargo\"] }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", branch = \"thread_rng\", version = \"~0.9\", default-features = false, features = [\"small_rng\", \"os_rng\"] }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", branch = \"0.3.x\", version = \"~0.4.28\", default-features = false, features = [\"kv_std\", \"kv_sval\"] }",
);

test_modules!(
    (rev, maximal_version, no_default_features, features),
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", rev = \"556820f\", version = \"<1.48\", default-features = false, features = [\"io_std\", \"io_utils\"] }",
    "clap = { git = \"https://github.com/clap-rs/clap\", rev = \"c7c761f988684ad97c8b2c521b05cf7f8192b3eb\", version = \"<4.5.50\", default-features = false, features = [\"derive\", \"cargo\"] }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", rev = \"db993ec\", version = \"<0.9\", default-features = false, features = [\"small_rng\", \"os_rng\"] }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", rev = \"6e1735597bb21c5d979a077395df85e1d633e077\", version = \"<0.4.28\", default-features = false, features = [\"kv_std\", \"kv_sval\"] }",
);

test_modules!(
    (rev, maximal_or_equal_version, no_default_features, features),
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", rev = \"556820f\", version = \"<=1.48\", default-features = false, features = [\"io_std\", \"io_utils\"] }",
    "clap = { git = \"https://github.com/clap-rs/clap\", rev = \"c7c761f988684ad97c8b2c521b05cf7f8192b3eb\", version = \"<=4.5.50\", default-features = false, features = [\"derive\", \"cargo\"] }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", rev = \"db993ec\", version = \"<=0.9\", default-features = false, features = [\"small_rng\", \"os_rng\"] }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", rev = \"6e1735597bb21c5d979a077395df85e1d633e077\", version = \"<=0.4.28\", default-features = false, features = [\"kv_std\", \"kv_sval\"] }",
);

test_modules!(
    (rev, minimal_version, no_default_features, features),
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", rev = \"556820f\", version = \">1.48\", default-features = false, features = [\"io_std\", \"io_utils\"] }",
    "clap = { git = \"https://github.com/clap-rs/clap\", rev = \"c7c761f988684ad97c8b2c521b05cf7f8192b3eb\", version = \">4.5.50\", default-features = false, features = [\"derive\", \"cargo\"] }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", rev = \"db993ec\", version = \">0.9\", default-features = false, features = [\"small_rng\", \"os_rng\"] }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", rev = \"6e1735597bb21c5d979a077395df85e1d633e077\", version = \">0.4.28\", default-features = false, features = [\"kv_std\", \"kv_sval\"] }",
);

test_modules!(
    (rev, minimal_or_equal_version, no_default_features, features),
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", rev = \"556820f\", version = \">=1.48\", default-features = false, features = [\"io_std\", \"io_utils\"] }",
    "clap = { git = \"https://github.com/clap-rs/clap\", rev = \"c7c761f988684ad97c8b2c521b05cf7f8192b3eb\", version = \">=4.5.50\", default-features = false, features = [\"derive\", \"cargo\"] }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", rev = \"db993ec\", version = \">=0.9\", default-features = false, features = [\"small_rng\", \"os_rng\"] }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", rev = \"6e1735597bb21c5d979a077395df85e1d633e077\", version = \">=0.4.28\", default-features = false, features = [\"kv_std\", \"kv_sval\"] }",
);

test_modules!(
    (rev, major_version, no_default_features, features),
    "tokio = { git = \"https://github.com/tokio-rs/tokio.git\", rev = \"556820f\", version = \"~1.48\", default-features = false, features = [\"io_std\", \"io_utils\"] }",
    "clap = { git = \"https://github.com/clap-rs/clap\", rev = \"c7c761f988684ad97c8b2c521b05cf7f8192b3eb\", version = \"~4.5.50\", default-features = false, features = [\"derive\", \"cargo\"] }",
    "rand = { git = \"ssh://git@github.com/rust-random/rand.git\", rev = \"db993ec\", version = \"~0.9\", default-features = false, features = [\"small_rng\", \"os_rng\"] }",
    "log = { git = \"ssh://git@github.com/rust-lang/log\", rev = \"6e1735597bb21c5d979a077395df85e1d633e077\", version = \"~0.4.28\", default-features = false, features = [\"kv_std\", \"kv_sval\"] }",
);

#[test]
fn could_not_parse() {
    let res = Dependency::from_str("http://localhost");
    assert!(res.is_err(), "{res:?}");
}
