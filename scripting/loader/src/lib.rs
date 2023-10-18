pub mod error;
pub mod loader;
pub mod parser;
pub mod transpiler;
pub use loader::Loader;

#[cfg(test)]
mod tests
{
    use crate::parser::*;
    use crate::transpiler::*;
    use fractal_gui::interface::Interactive;
    use std::fs;

    #[test]
    fn test_parse_input()
    {
        let content = fs::read_to_string("examples/sample_user_input.toml")
            .expect("Failed to read the TOML file");
        let user_input: UnparsedUserInput =
            toml::from_str(&content).expect("Failed to parse the TOML content");

        let _ = user_input.parse();
    }

    #[test]
    fn transpiler()
    {
        let content = fs::read_to_string("examples/sample_user_input.toml")
            .expect("Failed to read the TOML file");
        let user_input: UnparsedUserInput =
            toml::from_str(&content).expect("Failed to parse the TOML content");

        let transpiler = Transpiler::new(user_input);
        println!("{}", transpiler.gen_rust_profile());
    }

    #[test]
    fn parse_complex()
    {
        use num_complex::Complex64;
        use serde_json::Number;
        use serde_json::Value as JsonValue;
        let input0 = JsonValue::Number(Number::from(1));
        let input1 = JsonValue::String("3-2i".to_owned());
        let input2 = JsonValue::String("-6.283185307179586i".to_owned());
        let input3 = JsonValue::String("17".to_owned());
        let input4 = JsonValue::String("3- i".to_owned());

        let val0 = json_to_complex(&input0).unwrap();
        let val1 = json_to_complex(&input1).unwrap();
        let val2 = json_to_complex(&input2).unwrap();
        let val3 = json_to_complex(&input3).unwrap();
        let val4 = json_to_complex(&input4).unwrap();

        assert_eq!(val0, Complex64::new(1., 0.));
        assert_eq!(val1, Complex64::new(3., -2.));
        assert_eq!(val2, Complex64::new(0., -std::f64::consts::TAU));
        assert_eq!(val3, Complex64::new(17., 0.));
        assert_eq!(val4, Complex64::new(3., -1.));
    }

    #[test]
    fn loader()
    {
        use crate::loader::Loader;
        use std::path::Path;
        let toml_path = Path::new("../../user_scripts/.default.toml");
        // let output_path = Path::new("../output");
        let loader = Loader::new(toml_path, 768);

        unsafe {
            let int = loader.run().unwrap();
            assert_eq!(int.name(), "QuadRat Per(2, λ)");
        }

        let toml_path = Path::new("../../user_scripts/examples/mandelbrot.toml");
        let loader = Loader::new(toml_path, 768);

        unsafe {
            let int = loader.run().unwrap();
            assert_eq!(int.name(), "Mandelbrot");
        }
    }

    //     #[test]
    //     fn test_parse_dynamics()
    //     {
    //         let content = fs::read_to_string("examples/sample_user_input.toml")
    //             .expect("Failed to read the TOML file");
    //
    //         let user_input: UserInput =
    //             toml::from_str(&content).expect("Failed to parse the TOML content");
    //
    //         let context = user_input.parse_dynamics();
    //
    //         let test_python_code = r#"
    // a_real, a_imag = 10.0, 20.0  # Using separate real and imaginary parts
    // b_real, b_imag = 20.0, 30.0
    // z_real, z_imag = 3.0, 4.0
    //
    // a = complex(a_real, a_imag)
    // b = complex(b_real, b_imag)
    // z = complex(z_real, z_imag)
    //
    // # Call the parsed lambda function with specific values of the parameters and variable
    // result = f(a, b, z)
    //
    // (float(result.real), float(result.imag))
    // "#;
    //
    //         // Extract the lambda function from the context and execute Python code
    //         Python::with_gil(|py| {
    //             let f = context.get::<Py<PyAny>>("f").to_object(py);
    //
    //             let locals = [("f", f)].into_py_dict(py);
    //
    //             py.run(test_python_code, Some(locals), None)
    //                 .expect("Failed to execute Python code");
    //
    //             let result: &PyComplex = locals
    //                 .get_item("result")
    //                 .expect("Failed to get result")
    //                 .downcast()
    //                 .expect("Failed to downcast");
    //
    //             println!("Result: {}", result);
    //
    //             assert!((result.real() - 1.02318319859703).abs() < 1e-6);
    //             assert!((result.imag() - 0.00573163950553916).abs() < 1e-8);
    //         });
    //     }
}
