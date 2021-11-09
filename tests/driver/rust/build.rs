/* LICENSE BEGIN
    This file is part of the SixtyFPS Project -- https://sixtyfps.io
    Copyright (c) 2021 Olivier Goffart <olivier.goffart@sixtyfps.io>
    Copyright (c) 2021 Simon Hausmann <simon.hausmann@sixtyfps.io>

    SPDX-License-Identifier: GPL-3.0-only
    This file is also available under commercial licensing terms.
    Please contact info@sixtyfps.io for more information.
LICENSE END */
use std::io::Write;
use std::path::{Path, PathBuf};

use rayon::prelude::*;

fn main() -> std::io::Result<()> {
    let mut generated_file = std::fs::File::create(
        Path::new(&std::env::var_os("OUT_DIR").unwrap()).join("generated.rs"),
    )?;

    let compile_results: Vec<std::io::Result<Vec<PathBuf>>> =
        test_driver_lib::collect_test_cases()?
            .par_iter()
            .map(|testcase| {
                let mut dependencies = Vec::new();
                dependencies.push(testcase.absolute_path.clone());
                let mut module_name = testcase.identifier();
                if module_name.starts_with(|c: char| !c.is_ascii_alphabetic()) {
                    module_name.insert(0, '_');
                }
                let source = std::fs::read_to_string(&testcase.absolute_path)?;

                let mut output = std::fs::File::create(
                    Path::new(&std::env::var_os("OUT_DIR").unwrap())
                        .join(format!("{}.rs", module_name)),
                )?;

                #[cfg(not(feature = "build-time"))]
                if !generate_macro(&source, &mut output, testcase, &mut dependencies)? {
                    return Ok(dependencies);
                }
                #[cfg(feature = "build-time")]
                generate_source(&source, &mut output, testcase)?;

                for (i, x) in test_driver_lib::extract_test_functions(&source)
                    .filter(|x| x.language_id == "rust")
                    .enumerate()
                {
                    write!(
                        output,
                        r"
#[test] fn t_{}() -> Result<(), Box<dyn std::error::Error>> {{
    sixtyfps_rendering_backend_testing::init();
    {}
    Ok(())
}}",
                        i,
                        x.source.replace('\n', "\n    ")
                    )?;
                }

                Ok(dependencies)
            })
            .collect();

    for result in compile_results {
        let dependencies = result?;
        for dep in &dependencies {
            println!("cargo:rerun-if-changed={}", dep.display());
        }
    }

    for testcase in test_driver_lib::collect_test_cases()? {
        let mut module_name = testcase.identifier();
        if module_name.starts_with(|c: char| !c.is_ascii_alphabetic()) {
            module_name.insert(0, '_');
        }
        writeln!(generated_file, "#[path=\"{0}.rs\"] mod r#{0};", module_name)?;
    }

    // By default resources are embedded. The WASM example builds provide test coverage for that. This switch
    // provides test coverage for the non-embedding case, compiling tests without embedding the images.
    println!("cargo:rustc-env=SIXTYFPS_EMBED_RESOURCES=false");

    //Make sure to use a consistent style
    println!("cargo:rustc-env=SIXTYFPS_STYLE=fluent");

    Ok(())
}

#[cfg(not(feature = "build-time"))]
fn generate_macro(
    source: &str,
    output: &mut std::fs::File,
    testcase: &test_driver_lib::TestCase,
    dependencies: &mut Vec<PathBuf>,
) -> Result<bool, std::io::Error> {
    if source.contains("\\{") {
        // Unfortunately, \{ is not valid in a rust string so it cannot be used in a sixtyfps! macro
        output.write_all(b"#[test] #[ignore] fn ignored_because_string_template() {{}}")?;
        return Ok(false);
    }
    let include_paths = test_driver_lib::extract_include_paths(source);
    output.write_all(b"sixtyfps::sixtyfps!{")?;
    for path in include_paths {
        let mut abs_path = testcase.absolute_path.clone();
        abs_path.pop();
        abs_path.push(path);

        output.write_all(b"#[include_path=r#\"")?;
        output.write_all(abs_path.to_string_lossy().as_bytes())?;
        output.write_all(b"\"#]\n")?;

        //println!("cargo:rerun-if-changed={}", abs_path.to_string_lossy());
        dependencies.push(abs_path);
    }
    let mut abs_path = testcase.absolute_path.clone();
    abs_path.pop();
    output.write_all(b"#[include_path=r#\"")?;
    output.write_all(abs_path.to_string_lossy().as_bytes())?;
    output.write_all(b"\"#]\n")?;
    output.write_all(source.as_bytes())?;
    output.write_all(b"}\n")?;
    Ok(true)
}

#[cfg(feature = "build-time")]
fn generate_source(
    source: &str,
    output: &mut std::fs::File,
    testcase: &test_driver_lib::TestCase,
) -> Result<(), std::io::Error> {
    use sixtyfps_compilerlib::{diagnostics::BuildDiagnostics, *};

    let include_paths =
        test_driver_lib::extract_include_paths(source).map(PathBuf::from).collect::<Vec<_>>();

    let mut diag = BuildDiagnostics::default();
    let syntax_node = parser::parse(source.to_owned(), Some(&testcase.absolute_path), &mut diag);
    let mut compiler_config = CompilerConfiguration::new(generator::OutputFormat::Rust);
    compiler_config.include_paths = include_paths;
    let (root_component, mut diag) =
        spin_on::spin_on(compile_syntax_node(syntax_node, diag, compiler_config));

    if diag.has_error() {
        diag.print_warnings_and_exit_on_error();
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("build error in {:?}", testcase.absolute_path),
        ));
    }

    generator::generate(generator::OutputFormat::Rust, output, &root_component, &mut diag)?;
    diag.print_warnings_and_exit_on_error();
    Ok(())
}
