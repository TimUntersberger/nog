macro_rules! get_int {
    ($engine: ident, $ctx: ident, $scope: ident, $inputs: ident, $index: expr) => {
        $engine
            .eval_expression_tree($ctx, $scope, $inputs.get($index).unwrap())?
            .as_int()?
    };
}

macro_rules! get_string {
    ($engine: ident, $ctx: ident, $scope: ident, $inputs: ident, $index: expr) => {
        $engine
            .eval_expression_tree($ctx, $scope, $inputs.get($index).unwrap())?
            .as_str()?
            .to_string()
    };
}

macro_rules! get_dynamic {
    ($engine: ident, $ctx: ident, $scope: ident, $inputs: ident, $index: expr) => {
        $engine.eval_expression_tree($ctx, $scope, $inputs.get($index).unwrap())?
    };
}

macro_rules! get_map {
    ($engine: ident, $ctx: ident, $scope: ident, $inputs: ident, $index: expr) => {
        $engine
            .eval_expression_tree($ctx, $scope, $inputs.get($index).unwrap())?
            .cast::<rhai::Map>();
    };
}

macro_rules! get_type {
    ($engine: ident, $ctx: ident, $scope: ident, $inputs: ident, $index: expr, $type: ty) => {
        $engine
            .eval_expression_tree($ctx, $scope, $inputs.get($index).unwrap())?
            .cast::<$type>();
    };
}

macro_rules! get_variable_name {
    ($inputs: ident, $index: expr) => {
        $inputs
            .get($index)
            .unwrap()
            .get_variable_name()
            .unwrap()
            .to_string();
    };
}

macro_rules! set {
    ($typ: ty, $config: ident, $prop: ident, $key: ident, $val: ident) => {{
        if $key == stringify!($prop) {
            if $val.type_name().to_uppercase() != stringify!($typ).to_uppercase() {
                //TODO: change to EvalAtResult
                error!(
                    "{} has to be of type {} not {}",
                    stringify!($key),
                    stringify!($typ),
                    $val.type_name()
                );
            } else {
                $config.$prop = $val.clone().cast::<$typ>().into();
                continue;
            }
        }
    }};
}
