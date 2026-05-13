use wow_core::Position;

use crate::{
    MovementGenerator, MovementGeneratorFlags, MovementGeneratorMode, MovementGeneratorPriority,
    MovementGeneratorState, MovementGeneratorType,
};

pub const FLIGHT_TRAVEL_UPDATE_MS_LIKE_CPP: u32 = 100;
pub const FLIGHT_TIMEDIFF_NEXT_WP_MS_LIKE_CPP: u32 = 250;
pub const FLIGHT_SKIP_SPLINE_POINT_DISTANCE_SQ_LIKE_CPP: f32 = 40.0 * 40.0;
pub const PLAYER_FLIGHT_SPEED_LIKE_CPP: f32 = 32.0;
pub const TAXI_PATH_NODE_FLAG_TELEPORT_LIKE_CPP: u32 = 0x1;
pub const TAXI_PATH_NODE_FLAG_STOP_LIKE_CPP: u32 = 0x2;
pub const UNIT_STATE_IN_FLIGHT_LIKE_CPP: u32 = 0x0000_0100;
pub const UNIT_FLAG_REMOVE_CLIENT_CONTROL_LIKE_CPP: u32 = 0x0000_0004;
pub const UNIT_FLAG_ON_TAXI_LIKE_CPP: u32 = 0x0010_0000;
pub const PLAYER_FLAGS_TAXI_BENCHMARK_LIKE_CPP: u32 = 0x0002_0000;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TaxiPathNode {
    pub position: Position,
    pub id: u32,
    pub path_id: u16,
    pub node_index: i32,
    pub continent_id: u16,
    pub flags: u32,
    pub delay_ms: u32,
    pub arrival_event_id: u32,
    pub departure_event_id: u32,
}

impl TaxiPathNode {
    #[must_use]
    pub const fn new(
        path_id: u16,
        node_index: i32,
        continent_id: u16,
        x: f32,
        y: f32,
        z: f32,
    ) -> Self {
        Self {
            position: Position {
                x,
                y,
                z,
                orientation: 0.0,
            },
            id: 0,
            path_id,
            node_index,
            continent_id,
            flags: 0,
            delay_ms: 0,
            arrival_event_id: 0,
            departure_event_id: 0,
        }
    }

    #[must_use]
    pub const fn with_flags(mut self, flags: u32) -> Self {
        self.flags = flags;
        self
    }

    #[must_use]
    pub const fn with_delay(mut self, delay_ms: u32) -> Self {
        self.delay_ms = delay_ms;
        self
    }

    #[must_use]
    pub const fn with_events(mut self, departure_event_id: u32, arrival_event_id: u32) -> Self {
        self.departure_event_id = departure_event_id;
        self.arrival_event_id = arrival_event_id;
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TaxiPathSegment {
    pub path_id: u32,
    pub cost: u32,
    pub nodes: Vec<TaxiPathNode>,
}

impl TaxiPathSegment {
    #[must_use]
    pub fn new(path_id: u32, cost: u32, nodes: Vec<TaxiPathNode>) -> Self {
        Self {
            path_id,
            cost,
            nodes,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TaxiNodeChangeInfo {
    pub path_index: u32,
    pub cost: i64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlightLaunchPlan {
    pub path: Vec<Position>,
    pub first_point_id: u32,
    pub fly: bool,
    pub smooth: bool,
    pub uncompressed: bool,
    pub walk: bool,
    pub velocity: f32,
    pub set_unit_flags: u32,
    pub combat_stop_with_pets: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FlightEndGridInfo {
    pub map_id: u32,
    pub x: f32,
    pub y: f32,
    pub preload_target_node: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FlightPathEvent {
    pub event_id: u32,
    pub path_id: u16,
    pub node_index: i32,
    pub departure: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FlightPathSwitchAction {
    pub next_taxi_destination: bool,
    pub money_spent_criteria: Option<i64>,
    pub money_delta: Option<i64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlightUpdateAction {
    pub events: Vec<FlightPathEvent>,
    pub path_switches: Vec<FlightPathSwitchAction>,
    pub preload_grid: Option<FlightEndGridInfo>,
    pub finished: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FlightFinalizeContext {
    pub final_taxi_node: Option<Position>,
    pub final_taxi_node_map_id: Option<u32>,
    pub orientation: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FlightFinalizeAction {
    pub clear_taxi_destinations: bool,
    pub dismount: bool,
    pub remove_unit_flags: u32,
    pub stop_moving: bool,
    pub teleport: Option<(u32, Position)>,
    pub remove_player_flags: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FlightMovementAction {
    Continue,
    MissingPath,
    CurrentNodeAtEnd,
    Launch {
        launch: FlightLaunchPlan,
        end_grid: FlightEndGridInfo,
    },
    Update(FlightUpdateAction),
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlightPathMovementGenerator {
    state: MovementGeneratorState,
    path: Vec<TaxiPathNode>,
    current_node: u32,
    end_grid_x: f32,
    end_grid_y: f32,
    end_map_id: u32,
    preload_target_node: u32,
    points_for_path_switch: Vec<TaxiNodeChangeInfo>,
    pub finalize_action: Option<FlightFinalizeAction>,
}

impl Default for FlightPathMovementGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl FlightPathMovementGenerator {
    #[must_use]
    pub fn new() -> Self {
        Self {
            state: MovementGeneratorState {
                mode: MovementGeneratorMode::Default,
                priority: MovementGeneratorPriority::Highest,
                flags: MovementGeneratorFlags::INITIALIZATION_PENDING,
                base_unit_state: UNIT_STATE_IN_FLIGHT_LIKE_CPP,
            },
            path: Vec::new(),
            current_node: 0,
            end_grid_x: 0.0,
            end_grid_y: 0.0,
            end_map_id: 0,
            preload_target_node: 0,
            points_for_path_switch: Vec::new(),
            finalize_action: None,
        }
    }

    #[must_use]
    pub fn path(&self) -> &[TaxiPathNode] {
        &self.path
    }

    #[must_use]
    pub const fn current_node(&self) -> u32 {
        self.current_node
    }

    #[must_use]
    pub fn points_for_path_switch(&self) -> &[TaxiNodeChangeInfo] {
        &self.points_for_path_switch
    }

    #[must_use]
    pub fn has_arrived_like_cpp(&self) -> bool {
        self.current_node as usize >= self.path.len()
    }

    #[must_use]
    pub fn get_path_id_like_cpp(&self, index: usize) -> u32 {
        self.path
            .get(index)
            .map_or(0, |node| u32::from(node.path_id))
    }

    #[must_use]
    pub fn reset_position_like_cpp(&self) -> Option<Position> {
        self.path
            .get(self.current_node as usize)
            .map(|node| node.position)
    }

    pub fn load_path_from_segments_like_cpp(
        &mut self,
        segments: &[TaxiPathSegment],
        discount: f32,
        start_node: u32,
    ) {
        self.path.clear();
        self.current_node = start_node;
        self.points_for_path_switch.clear();

        for (src, segment) in segments.iter().enumerate() {
            if !segment.nodes.is_empty() {
                let start = segment.nodes[0];
                let end = segment.nodes[segment.nodes.len() - 1];
                let segment_is_last = src == segments.len() - 1;
                let mut passed_previous_segment_proximity_check = false;
                for (i, node) in segment.nodes.iter().copied().enumerate() {
                    if passed_previous_segment_proximity_check
                        || src == 0
                        || self.path.is_empty()
                        || is_node_included_in_shortened_path_like_cpp(
                            self.path.last().expect("path checked above"),
                            &node,
                        )
                    {
                        if (src == 0
                            || (is_node_included_in_shortened_path_like_cpp(&start, &node)
                                && i >= 2))
                            && (segment_is_last
                                || (is_node_included_in_shortened_path_like_cpp(&end, &node)
                                    && (i < segment.nodes.len() - 1 || self.path.is_empty())))
                            && (!node_has_flag(node, TAXI_PATH_NODE_FLAG_TELEPORT_LIKE_CPP)
                                || self.path.is_empty()
                                || !node_has_flag(
                                    *self.path.last().expect("path checked above"),
                                    TAXI_PATH_NODE_FLAG_TELEPORT_LIKE_CPP,
                                ))
                        {
                            passed_previous_segment_proximity_check = true;
                            self.path.push(node);
                        }
                    } else {
                        self.path.pop();
                        if let Some(last_switch) = self.points_for_path_switch.last_mut() {
                            last_switch.path_index = last_switch.path_index.saturating_sub(1);
                        }
                    }
                }
            }

            self.points_for_path_switch.push(TaxiNodeChangeInfo {
                path_index: self.path.len().max(1) as u32 - 1,
                cost: f64::from(segment.cost) // use f64 after multiplying to preserve ceil shape.
                    .mul_add(f64::from(discount), 0.0)
                    .ceil() as i64,
            });
        }
    }

    pub fn initialize_like_cpp(&mut self, owner_position: Position) -> FlightMovementAction {
        self.remove_flag(
            MovementGeneratorFlags::INITIALIZATION_PENDING | MovementGeneratorFlags::DEACTIVATED,
        );
        self.add_flag(MovementGeneratorFlags::INITIALIZED);

        let reset = self.reset_like_cpp(owner_position);
        if matches!(
            reset,
            FlightMovementAction::MissingPath | FlightMovementAction::CurrentNodeAtEnd
        ) {
            return reset;
        }

        let end_grid = self.init_end_grid_info_like_cpp();
        match reset {
            FlightMovementAction::Launch { launch, .. } => {
                FlightMovementAction::Launch { launch, end_grid }
            }
            other => other,
        }
    }

    pub fn reset_like_cpp(&mut self, owner_position: Position) -> FlightMovementAction {
        self.remove_flag(MovementGeneratorFlags::DEACTIVATED);

        if self.path.is_empty() {
            return FlightMovementAction::MissingPath;
        }

        let end = self.get_path_at_map_end_like_cpp();
        let current_node_id = self.current_node;
        if current_node_id == end {
            return FlightMovementAction::CurrentNodeAtEnd;
        }

        let mut path = Vec::with_capacity((end - current_node_id + 1) as usize);
        path.push(owner_position);
        for index in current_node_id..end {
            path.push(self.path[index as usize].position);
        }

        FlightMovementAction::Launch {
            launch: FlightLaunchPlan {
                path,
                first_point_id: current_node_id,
                fly: true,
                smooth: true,
                uncompressed: true,
                walk: true,
                velocity: PLAYER_FLIGHT_SPEED_LIKE_CPP,
                set_unit_flags: UNIT_FLAG_REMOVE_CLIENT_CONTROL_LIKE_CPP
                    | UNIT_FLAG_ON_TAXI_LIKE_CPP,
                combat_stop_with_pets: true,
            },
            end_grid: self.end_grid_info(),
        }
    }

    pub fn update_like_cpp(&mut self, current_path_idx: i32) -> FlightMovementAction {
        let point_id = if current_path_idx <= 0 {
            0
        } else {
            current_path_idx as u32 - 1
        };

        let mut action = FlightUpdateAction {
            events: Vec::new(),
            path_switches: Vec::new(),
            preload_grid: None,
            finished: false,
        };

        if point_id > self.current_node
            && (self.current_node as usize) < self.path.len().saturating_sub(1)
        {
            let mut departure_event = true;
            while (self.current_node as usize) < self.path.len().saturating_sub(1) {
                let node = self.path[self.current_node as usize];
                if let Some(event) = flight_event_like_cpp(node, departure_event) {
                    action.events.push(event);
                }

                while self
                    .points_for_path_switch
                    .first()
                    .is_some_and(|switch| switch.path_index <= self.current_node)
                {
                    self.points_for_path_switch.remove(0);
                    let next_cost = self
                        .points_for_path_switch
                        .first()
                        .map(|switch| switch.cost);
                    action.path_switches.push(FlightPathSwitchAction {
                        next_taxi_destination: true,
                        money_spent_criteria: next_cost,
                        money_delta: next_cost.map(|cost| -cost),
                    });
                }

                if point_id == self.current_node {
                    break;
                }

                if self.current_node == self.preload_target_node {
                    action.preload_grid = Some(self.end_grid_info());
                }

                if departure_event {
                    self.current_node += 1;
                }
                departure_event = !departure_event;
            }
        }

        if self.current_node as usize >= self.path.len().saturating_sub(1) {
            self.add_flag(MovementGeneratorFlags::INFORM_ENABLED);
            action.finished = true;
        }

        FlightMovementAction::Update(action)
    }

    #[must_use]
    pub fn get_path_at_map_end_like_cpp(&self) -> u32 {
        if self.current_node as usize >= self.path.len() {
            return self.path.len() as u32;
        }

        let current_map_id = self.path[self.current_node as usize].continent_id;
        for index in self.current_node as usize..self.path.len() {
            if self.path[index].continent_id != current_map_id {
                return index as u32;
            }
            if index > 0
                && node_has_flag(self.path[index - 1], TAXI_PATH_NODE_FLAG_TELEPORT_LIKE_CPP)
            {
                return index as u32;
            }
        }

        self.path.len() as u32
    }

    pub fn set_current_node_after_teleport_like_cpp(&mut self) {
        if self.path.is_empty() || self.current_node as usize >= self.path.len() {
            return;
        }

        let map0 = self.path[self.current_node as usize].continent_id;
        for index in self.current_node as usize + 1..self.path.len() {
            if self.path[index].continent_id != map0
                || node_has_flag(self.path[index - 1], TAXI_PATH_NODE_FLAG_TELEPORT_LIKE_CPP)
            {
                self.current_node = index as u32;
                return;
            }
        }
    }

    pub fn skip_current_node_like_cpp(&mut self) {
        self.current_node = self.current_node.saturating_add(1);
    }

    pub fn deactivate_like_cpp(&mut self) {
        self.add_flag(MovementGeneratorFlags::DEACTIVATED);
    }

    pub fn finalize_like_cpp(
        &mut self,
        active: bool,
        context: FlightFinalizeContext,
    ) -> Option<FlightFinalizeAction> {
        self.add_flag(MovementGeneratorFlags::FINALIZED);
        if !active {
            return None;
        }

        let should_teleport_to_final = !self.path.is_empty()
            && (self.path.len() < 2
                || !node_has_flag(
                    self.path[self.path.len() - 2],
                    TAXI_PATH_NODE_FLAG_TELEPORT_LIKE_CPP,
                ));
        let teleport = should_teleport_to_final
            .then_some(())
            .and_then(|()| context.final_taxi_node.zip(context.final_taxi_node_map_id))
            .map(|(position, map_id)| {
                (
                    map_id,
                    Position::new(position.x, position.y, position.z, context.orientation),
                )
            });

        let action = FlightFinalizeAction {
            clear_taxi_destinations: true,
            dismount: true,
            remove_unit_flags: UNIT_FLAG_REMOVE_CLIENT_CONTROL_LIKE_CPP
                | UNIT_FLAG_ON_TAXI_LIKE_CPP,
            stop_moving: true,
            teleport,
            remove_player_flags: PLAYER_FLAGS_TAXI_BENCHMARK_LIKE_CPP,
        };
        self.finalize_action = Some(action);
        Some(action)
    }

    fn init_end_grid_info_like_cpp(&mut self) -> FlightEndGridInfo {
        let node_count = self.path.len();
        assert!(
            node_count > 0,
            "FlightPathMovementGenerator::InitEndGridInfo called with empty path"
        );
        self.end_map_id = u32::from(self.path[node_count - 1].continent_id);
        self.preload_target_node = if node_count < 3 {
            0
        } else {
            node_count as u32 - 3
        };

        while self.path[self.preload_target_node as usize].continent_id as u32 != self.end_map_id {
            self.preload_target_node += 1;
        }

        self.end_grid_x = self.path[node_count - 1].position.x;
        self.end_grid_y = self.path[node_count - 1].position.y;
        self.end_grid_info()
    }

    const fn end_grid_info(&self) -> FlightEndGridInfo {
        FlightEndGridInfo {
            map_id: self.end_map_id,
            x: self.end_grid_x,
            y: self.end_grid_y,
            preload_target_node: self.preload_target_node,
        }
    }
}

impl MovementGenerator for FlightPathMovementGenerator {
    fn state(&self) -> &MovementGeneratorState {
        &self.state
    }

    fn state_mut(&mut self) -> &mut MovementGeneratorState {
        &mut self.state
    }

    fn kind(&self) -> MovementGeneratorType {
        MovementGeneratorType::Flight
    }

    fn initialize(&mut self) {
        self.remove_flag(
            MovementGeneratorFlags::INITIALIZATION_PENDING | MovementGeneratorFlags::DEACTIVATED,
        );
        self.add_flag(MovementGeneratorFlags::INITIALIZED);
    }

    fn reset(&mut self) {
        self.remove_flag(MovementGeneratorFlags::DEACTIVATED);
    }

    fn update(&mut self, _diff_ms: u32) -> bool {
        true
    }

    fn deactivate(&mut self) {
        self.deactivate_like_cpp();
    }

    fn finalize(&mut self, active: bool, _movement_inform: bool) {
        let _ = self.finalize_like_cpp(
            active,
            FlightFinalizeContext {
                final_taxi_node: None,
                final_taxi_node_map_id: None,
                orientation: 0.0,
            },
        );
    }

    fn reset_position(&self) -> Option<(f32, f32, f32)> {
        self.reset_position_like_cpp()
            .map(|position| (position.x, position.y, position.z))
    }
}

#[must_use]
pub fn is_node_included_in_shortened_path_like_cpp(p1: &TaxiPathNode, p2: &TaxiPathNode) -> bool {
    p1.continent_id != p2.continent_id
        || {
            let dx = p1.position.x - p2.position.x;
            let dy = p1.position.y - p2.position.y;
            dx * dx + dy * dy > FLIGHT_SKIP_SPLINE_POINT_DISTANCE_SQ_LIKE_CPP
        }
        || node_has_flag(*p2, TAXI_PATH_NODE_FLAG_TELEPORT_LIKE_CPP)
        || (node_has_flag(*p2, TAXI_PATH_NODE_FLAG_STOP_LIKE_CPP) && p2.delay_ms != 0)
}

fn flight_event_like_cpp(node: TaxiPathNode, departure: bool) -> Option<FlightPathEvent> {
    let event_id = if departure {
        node.departure_event_id
    } else {
        node.arrival_event_id
    };
    (event_id != 0).then_some(FlightPathEvent {
        event_id,
        path_id: node.path_id,
        node_index: node.node_index,
        departure,
    })
}

const fn node_has_flag(node: TaxiPathNode, flag: u32) -> bool {
    node.flags & flag != 0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pos(x: f32, y: f32, z: f32) -> Position {
        Position::new(x, y, z, 1.25)
    }

    fn node(index: i32, map: u16, x: f32) -> TaxiPathNode {
        TaxiPathNode::new(42, index, map, x, 0.0, 10.0)
    }

    fn node_pos(x: f32) -> Position {
        Position::new(x, 0.0, 10.0, 0.0)
    }

    #[test]
    fn flight_constructor_matches_cpp_shape() {
        let flight = FlightPathMovementGenerator::new();
        assert_eq!(flight.kind(), MovementGeneratorType::Flight);
        assert_eq!(flight.state().mode, MovementGeneratorMode::Default);
        assert_eq!(flight.state().priority, MovementGeneratorPriority::Highest);
        assert_eq!(
            flight.state().flags,
            MovementGeneratorFlags::INITIALIZATION_PENDING
        );
        assert_eq!(
            flight.state().base_unit_state,
            UNIT_STATE_IN_FLIGHT_LIKE_CPP
        );
        assert_eq!(flight.current_node(), 0);
        assert!(flight.path().is_empty());
    }

    #[test]
    fn flight_path_at_map_end_stops_on_map_change_and_teleport_previous_node() {
        let mut flight = FlightPathMovementGenerator::new();
        flight.path = vec![
            node(0, 1, 0.0),
            node(1, 1, 50.0),
            node(2, 2, 100.0),
            node(3, 2, 150.0),
        ];
        assert_eq!(flight.get_path_at_map_end_like_cpp(), 2);

        flight.path = vec![
            node(0, 1, 0.0),
            node(1, 1, 50.0).with_flags(TAXI_PATH_NODE_FLAG_TELEPORT_LIKE_CPP),
            node(2, 1, 100.0),
        ];
        assert_eq!(flight.get_path_at_map_end_like_cpp(), 2);
    }

    #[test]
    fn flight_shortened_path_inclusion_matches_cpp_conditions() {
        let near = node(0, 1, 0.0);
        assert!(!is_node_included_in_shortened_path_like_cpp(
            &near,
            &node(1, 1, 10.0)
        ));
        assert!(is_node_included_in_shortened_path_like_cpp(
            &near,
            &node(1, 1, 41.0)
        ));
        assert!(is_node_included_in_shortened_path_like_cpp(
            &near,
            &node(1, 2, 10.0)
        ));
        assert!(is_node_included_in_shortened_path_like_cpp(
            &near,
            &node(1, 1, 10.0).with_flags(TAXI_PATH_NODE_FLAG_TELEPORT_LIKE_CPP)
        ));
        assert!(is_node_included_in_shortened_path_like_cpp(
            &near,
            &node(1, 1, 10.0)
                .with_flags(TAXI_PATH_NODE_FLAG_STOP_LIKE_CPP)
                .with_delay(10)
        ));
    }

    #[test]
    fn flight_load_path_shortens_segments_and_records_switch_costs() {
        let mut flight = FlightPathMovementGenerator::new();
        let segments = vec![
            TaxiPathSegment::new(
                42,
                101,
                vec![node(0, 1, 0.0), node(1, 1, 10.0), node(2, 1, 60.0)],
            ),
            TaxiPathSegment::new(
                43,
                199,
                vec![node(0, 1, 60.0), node(1, 1, 65.0), node(2, 1, 120.0)],
            ),
        ];
        flight.load_path_from_segments_like_cpp(&segments, 0.8, 0);
        assert_eq!(
            flight
                .path()
                .iter()
                .map(|n| n.position.x)
                .collect::<Vec<_>>(),
            vec![0.0, 10.0, 120.0]
        );
        assert_eq!(
            flight.points_for_path_switch(),
            &[
                TaxiNodeChangeInfo {
                    path_index: 1,
                    cost: 81,
                },
                TaxiNodeChangeInfo {
                    path_index: 2,
                    cost: 160,
                },
            ]
        );
    }

    #[test]
    fn flight_initialize_launches_spline_plan_and_end_grid_like_cpp() {
        let mut flight = FlightPathMovementGenerator::new();
        flight.path = vec![node(0, 1, 10.0), node(1, 1, 50.0), node(2, 1, 90.0)];
        let action = flight.initialize_like_cpp(pos(0.0, 0.0, 5.0));
        let FlightMovementAction::Launch { launch, end_grid } = action else {
            panic!("expected launch");
        };
        assert_eq!(
            launch.path,
            vec![
                pos(0.0, 0.0, 5.0),
                node_pos(10.0),
                node_pos(50.0),
                node_pos(90.0),
            ]
        );
        assert_eq!(launch.first_point_id, 0);
        assert!(launch.fly);
        assert!(launch.smooth);
        assert!(launch.uncompressed);
        assert!(launch.walk);
        assert_eq!(launch.velocity, PLAYER_FLIGHT_SPEED_LIKE_CPP);
        assert_eq!(
            launch.set_unit_flags,
            UNIT_FLAG_REMOVE_CLIENT_CONTROL_LIKE_CPP | UNIT_FLAG_ON_TAXI_LIKE_CPP
        );
        assert!(launch.combat_stop_with_pets);
        assert_eq!(
            end_grid,
            FlightEndGridInfo {
                map_id: 1,
                x: 90.0,
                y: 0.0,
                preload_target_node: 0,
            }
        );
    }

    #[test]
    fn flight_update_alternates_departure_arrival_switches_costs_and_finishes() {
        let mut flight = FlightPathMovementGenerator::new();
        flight.path = vec![
            node(0, 1, 0.0).with_events(10, 11),
            node(1, 1, 50.0).with_events(20, 21),
            node(2, 1, 90.0).with_events(30, 31),
        ];
        flight.points_for_path_switch = vec![
            TaxiNodeChangeInfo {
                path_index: 0,
                cost: 50,
            },
            TaxiNodeChangeInfo {
                path_index: 1,
                cost: 75,
            },
        ];
        flight.init_end_grid_info_like_cpp();

        let FlightMovementAction::Update(action) = flight.update_like_cpp(3) else {
            panic!("expected update");
        };
        assert_eq!(
            action.events,
            vec![
                FlightPathEvent {
                    event_id: 10,
                    path_id: 42,
                    node_index: 0,
                    departure: true,
                },
                FlightPathEvent {
                    event_id: 21,
                    path_id: 42,
                    node_index: 1,
                    departure: false,
                },
                FlightPathEvent {
                    event_id: 20,
                    path_id: 42,
                    node_index: 1,
                    departure: true,
                },
            ]
        );
        assert_eq!(
            action.path_switches,
            vec![
                FlightPathSwitchAction {
                    next_taxi_destination: true,
                    money_spent_criteria: Some(75),
                    money_delta: Some(-75),
                },
                FlightPathSwitchAction {
                    next_taxi_destination: true,
                    money_spent_criteria: None,
                    money_delta: None,
                },
            ]
        );
        assert!(action.finished);
        assert!(flight.has_flag(MovementGeneratorFlags::INFORM_ENABLED));
    }

    #[test]
    fn flight_teleport_and_finalize_match_cpp_branches() {
        let mut flight = FlightPathMovementGenerator::new();
        flight.path = vec![
            node(0, 1, 0.0),
            node(1, 1, 50.0).with_flags(TAXI_PATH_NODE_FLAG_TELEPORT_LIKE_CPP),
            node(2, 2, 90.0),
        ];
        flight.current_node = 0;
        flight.set_current_node_after_teleport_like_cpp();
        assert_eq!(flight.current_node(), 2);
        flight.skip_current_node_like_cpp();
        assert_eq!(flight.current_node(), 3);

        let action = flight.finalize_like_cpp(
            true,
            FlightFinalizeContext {
                final_taxi_node: Some(pos(200.0, 1.0, 2.0)),
                final_taxi_node_map_id: Some(571),
                orientation: 2.0,
            },
        );
        assert_eq!(
            action,
            Some(FlightFinalizeAction {
                clear_taxi_destinations: true,
                dismount: true,
                remove_unit_flags: UNIT_FLAG_REMOVE_CLIENT_CONTROL_LIKE_CPP
                    | UNIT_FLAG_ON_TAXI_LIKE_CPP,
                stop_moving: true,
                teleport: None,
                remove_player_flags: PLAYER_FLAGS_TAXI_BENCHMARK_LIKE_CPP,
            })
        );

        let mut no_teleport_before_final = FlightPathMovementGenerator::new();
        no_teleport_before_final.path = vec![node(0, 1, 0.0), node(1, 1, 50.0)];
        assert_eq!(
            no_teleport_before_final
                .finalize_like_cpp(
                    true,
                    FlightFinalizeContext {
                        final_taxi_node: Some(pos(200.0, 1.0, 2.0)),
                        final_taxi_node_map_id: Some(571),
                        orientation: 2.0,
                    },
                )
                .expect("active finalize")
                .teleport,
            Some((571, Position::new(200.0, 1.0, 2.0, 2.0)))
        );
    }
}
