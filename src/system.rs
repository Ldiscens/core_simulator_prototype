
#[derive(Debug, Default)]
pub struct System {
    pub reactivity: f64,
    pub effective_beta: f64,
    pub generation_time: f64,
    pub k: f64,
    pub neutron_pop: f64,
    pub precursors_proportion: [f64; 6],
    pub precursors_lambda: [f64; 6],
    pub precursors_pop: [f64; 6],
    pub rods: f64,
    pub boric_acid: f64,
    pub depletion: f64,
    pub dt: f64,
    pub current_time: f64,
    pub external_source_activity: f64,    
    pub is_scraming: bool,
    pub external_source_is_in: bool,    
    pub doubling_time_estimator: f64,
    pub power_estimator: f64,
}

pub fn reset_system(system: &mut System) {
    system.reactivity = 0.0e-5;
    system.effective_beta = 650e-5;
    system.generation_time = 1e-4;
    system.k = 1.0;
    system.neutron_pop = 1e5;
    system.precursors_proportion = [0.033, 0.219, 0.196, 0.395, 0.115, 0.042];
    system.precursors_lambda = [1.0/55.9, 1.0/22.7, 1.0/6.24, 1.0/2.3, 1.0/0.61, 1.0/0.23];
    system.precursors_pop = [0.0; 6];
    system.rods = -5500.0e-5;
    system.boric_acid = -15000.0e-5;
    system.depletion = 20000.0e-5;
    system.dt = 1e-5;
    system.current_time = 0.0;
    system.is_scraming = false;
    system.external_source_is_in = true;
    system.external_source_activity = 6.335e11 * 0.1; //100mg of Pu238
    system.doubling_time_estimator = 0.0;
    system.power_estimator = 0.0;
}


pub fn next_dt(system: &mut System) {
    let old_system = System {
        ..*system
    };

    let mut sum_lambda_f_p_f = 0.0;
    for i in 0..old_system.precursors_pop.len() {
        sum_lambda_f_p_f = sum_lambda_f_p_f + old_system.precursors_lambda[i]*old_system.precursors_pop[i];
    }
    let contrib_prompt = (old_system.reactivity - old_system.effective_beta)*old_system.k*old_system.neutron_pop/old_system.generation_time;
    // println!("sum_lambda_f_p_f = {sum_lambda_f_p_f} ; contrib_prompt = {contrib_prompt}");

    system.neutron_pop = old_system.neutron_pop + system.dt*(contrib_prompt + sum_lambda_f_p_f);
    if system.external_source_is_in {
        system.neutron_pop += system.external_source_activity*system.dt;
    }
    if system.neutron_pop < 0.0 {
        system.neutron_pop = 0.0;
    }

    for i in 0..old_system.precursors_pop.len() {
        system.precursors_pop[i] = old_system.precursors_pop[i] + system.dt*( old_system.effective_beta*old_system.precursors_proportion[i]*old_system.k*old_system.neutron_pop/old_system.generation_time - old_system.precursors_lambda[i]*old_system.precursors_pop[i]);
    }
    system.k = system.neutron_pop / old_system.neutron_pop;
    system.reactivity = (system.k - 1.0)/system.k + system.rods + system.boric_acid + system.depletion;
    system.current_time += system.dt;
}