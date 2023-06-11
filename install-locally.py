import os

if __name__ == "__main__":
    # Build bindings
    os.system("maturin build --release -m crates/bindings/Cargo.toml")

    # Uninstall old version
    os.system("pip uninstall --disable-pip-version-check -y chainner_rs")

    # Install new version
    wheels = os.listdir("target/wheels")
    os.system(f"pip install --disable-pip-version-check target/wheels/{wheels[0]}")
