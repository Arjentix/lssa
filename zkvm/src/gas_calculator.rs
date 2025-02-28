#[derive(Debug, Clone)]
pub struct GasCalculator {
    /// Gas spent per deploying one byte of data
    gas_fee_per_byte_deploy: u64,
    /// Gas spent per reading one byte of data in VM
    gas_fee_per_input_buffer_runtime: u64,
    /// Gas spent per one byte of contract data in runtime
    gas_fee_per_byte_runtime: u64,
    /// Cost of one gas of runtime in public balance
    gas_cost_runtime: u64,
    /// Cost of one gas of deployment in public balance
    gas_cost_deploy: u64,
    /// Gas limit for deployment
    gas_limit_deploy: u64,
    /// Gas limit for runtime
    gas_limit_runtime: u64,
}

impl GasCalculator {
    pub fn new(
        gas_fee_per_byte_deploy: u64,
        gas_fee_per_input_buffer_runtime: u64,
        gas_fee_per_byte_runtime: u64,
        gas_cost_runtime: u64,
        gas_cost_deploy: u64,
        gas_limit_deploy: u64,
        gas_limit_runtime: u64,
    ) -> Self {
        Self {
            gas_fee_per_byte_deploy,
            gas_fee_per_input_buffer_runtime,
            gas_fee_per_byte_runtime,
            gas_cost_deploy,
            gas_cost_runtime,
            gas_limit_deploy,
            gas_limit_runtime,
        }
    }

    pub fn gas_fee_per_byte_deploy(&self) -> u64 {
        self.gas_fee_per_byte_deploy
    }

    pub fn gas_fee_per_input_buffer_runtime(&self) -> u64 {
        self.gas_fee_per_input_buffer_runtime
    }

    pub fn gas_fee_per_byte_runtime(&self) -> u64 {
        self.gas_fee_per_byte_runtime
    }

    pub fn gas_cost_runtime(&self) -> u64 {
        self.gas_cost_runtime
    }

    pub fn gas_cost_deploy(&self) -> u64 {
        self.gas_cost_deploy
    }

    pub fn gas_limit_deploy(&self) -> u64 {
        self.gas_limit_deploy
    }

    pub fn gas_limit_runtime(&self) -> u64 {
        self.gas_limit_runtime
    }

    ///Returns Option<u64>
    ///
    /// Some(_) - in case if `gas` < `gas_limit_deploy`
    ///
    /// None - else
    pub fn gas_deploy(&self, elf: &[u8]) -> Option<u64> {
        let gas = self.gas_fee_per_byte_deploy() * (elf.len() as u64);

        if gas < self.gas_limit_deploy() {
            Some(gas)
        } else {
            None
        }
    }

    pub fn gas_runtime(&self, elf: &[u8]) -> u64 {
        self.gas_fee_per_byte_runtime() * (elf.len() as u64)
    }

    pub fn gas_input_buffer(&self, input_length: usize) -> u64 {
        self.gas_fee_per_input_buffer_runtime() * (input_length as u64)
    }

    ///Returns Option<u64>
    ///
    /// Some(_) - in case if `gas` < `gas_limit_runtime`
    ///
    /// None - else
    pub fn gas_runtime_full(&self, elf: &[u8], input_length: usize) -> Option<u64> {
        let gas = self.gas_runtime(elf) + self.gas_input_buffer(input_length);

        if gas < self.gas_limit_runtime() {
            Some(gas)
        } else {
            None
        }
    }

    pub fn deploy_cost(&self, deploy_gas: u64) -> u64 {
        deploy_gas * self.gas_cost_deploy()
    }

    pub fn runtime_cost(&self, runtime_gas: u64) -> u64 {
        runtime_gas * self.gas_cost_runtime()
    }
}
