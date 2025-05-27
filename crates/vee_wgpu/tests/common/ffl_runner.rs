use glam::Mat4;
use std::{
    error::Error
    ,
    path::PathBuf,
    process::{Command, Output},
};

pub struct FFlRunner {
    pub dir: PathBuf,
}

impl FFlRunner {
    pub fn run_ffl_testing(&mut self) -> Result<Output, Box<dyn Error>> {
        Command::new("cmake")
            .args(["-S", ".", "-B", "build"])
            .current_dir(&self.dir)
            .spawn()?
            .wait()?;

        Command::new("cmake")
            .args(["--build", "build"])
            .current_dir(&self.dir)
            .spawn()?
            .wait()?;

        let exit = Command::new("timeout")
            .args(["2", "./ffl_testing_2"])
            .current_dir(&self.dir)
            .spawn()?
            .wait_with_output()?;

        Ok(exit)
    }

    pub fn get_resultant_file(&mut self, file: &str) -> Result<String, Box<dyn Error>> {
        Ok(std::fs::read_to_string(self.dir.join(file))?)
    }

    pub fn get_resultant_mtx44(&mut self, file: &str) -> Result<Mat4, Box<dyn Error>> {
        let v: Vec<f32> = self
            .get_resultant_file(file)?
            .lines()
            .map(|str| str.parse::<f32>())
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Mat4::from_cols_slice(&v).transpose())
    }
}
