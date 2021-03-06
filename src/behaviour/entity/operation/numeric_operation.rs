use std::convert::AsRef;
use std::sync::{Arc, RwLock};

use log::debug;
use serde_json::{json, Value};

use crate::behaviour::entity::operation::NumericOperationFunction;
use crate::frp::Stream;
use crate::model::{PropertyInstanceGetter, PropertyInstanceSetter, ReactiveEntityInstance};
use crate::reactive::entity::operation::Operation;
use crate::reactive::entity::Disconnectable;
use crate::NumericOperationProperties;

pub const NUMERIC_OPERATION: &str = "numeric_operation";

/// Generic implementation of numeric operations with one input and one result.
///
/// The implementation is realized using reactive streams.
pub struct NumericOperation<'a> {
    pub f: NumericOperationFunction<f64>,

    pub internal_result: RwLock<Stream<'a, Value>>,

    pub entity: Arc<ReactiveEntityInstance>,

    pub handle_id: u128,
}

impl NumericOperation<'_> {
    pub fn new(e: Arc<ReactiveEntityInstance>, f: NumericOperationFunction<f64>) -> NumericOperation<'static> {
        let handle_id = e.properties.get(NumericOperationProperties::RESULT.as_ref()).unwrap().id.as_u128();

        let internal_result = e
            .properties
            .get(NumericOperationProperties::LHS.as_ref())
            .unwrap()
            .stream
            .read()
            .unwrap()
            .map(move |v| json!(f(v.as_f64().unwrap())));
        let numeric_operation = NumericOperation {
            f,
            internal_result: RwLock::new(internal_result),
            entity: e.clone(),
            handle_id,
        };

        // Connect the internal result with the stream of the result property
        numeric_operation.internal_result.read().unwrap().observe_with_handle(
            move |v| {
                debug!("Setting result of {}: {}", NUMERIC_OPERATION, v);
                e.set(NumericOperationProperties::RESULT.to_string(), json!(*v));
            },
            handle_id,
        );

        numeric_operation
    }

    /// TODO: extract to trait "Named"
    /// TODO: unit test
    pub fn type_name(&self) -> String {
        self.entity.type_name.clone()
    }
}

impl Disconnectable for NumericOperation<'_> {
    /// TODO: Add guard: disconnect only if actually connected
    fn disconnect(&self) {
        debug!("Disconnect {} {} with handle {}", NUMERIC_OPERATION, self.type_name(), self.handle_id);
        self.internal_result.read().unwrap().remove(self.handle_id);
    }
}

impl Operation for NumericOperation<'_> {
    fn lhs(&self, value: Value) {
        self.entity.set(NumericOperationProperties::LHS.as_ref(), value);
    }

    fn result(&self) -> Value {
        self.entity.get(NumericOperationProperties::RESULT.as_ref()).unwrap()
    }
}

/// Automatically disconnect streams on destruction
impl Drop for NumericOperation<'_> {
    fn drop(&mut self) {
        self.disconnect();
    }
}
