fn main()
{
    #[cfg(feature = "scripting")]
    {
        // Get Python library directory from python3-config
        if let Ok(output) = std::process::Command::new("python3-config")
            .arg("--ldflags")
            .output()
            && output.status.success()
        {
            let ldflags = String::from_utf8_lossy(&output.stdout);

            // Extract library path from -L flag
            for path in ldflags
                .split_whitespace()
                .filter_map(|flag| flag.strip_prefix("-L"))
            {
                println!("cargo:rustc-link-search=native={}", path);
                // Add rpath so the binary can find the library at runtime
                println!("cargo:rustc-link-arg=-Wl,-rpath,{}", path);
            }
        }
    }
}
