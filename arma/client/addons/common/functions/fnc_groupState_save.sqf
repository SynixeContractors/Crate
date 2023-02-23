#include "script_component.hpp"

params ["_group"];

private _state = createHashMap;

_state set ["id", groupId _group];
_state set ["leader", (leader _group) getVariable [QGVAR(id), -1]];
_state set ["behaviour", combatBehaviour _group];
_state set ["side", side _group];

private _speedMode = speedMode _group;
if (_speedMode != "NORMAL") then {
    _state set ["speed", _speedMode];
};

private _combatMode = combatMode _group;
if (_combatMode != "YELLOW") then {
    _state set ["combat", _combatMode];
};

private _formation = formation _group;
if (_formation != "WEDGE") then {
    _state set ["formation", _formation];
};

private _waypoints = waypoints _group;
if (count _waypoints > 1) then {
    private _points = [];
    {
        if (_forEachIndex != 0) then {
            private _waypointVars = createHashMap;
            if !(wayPointVisible _x) then {
                _waypointVars set ["visible", false];
            };
            if (waypointDescription _x isNotEqualTo "") then {
                _waypointVars set ["description", waypointDescription _x];
            };
            if (waypointSpeed _x isNotEqualTo "UNCHANGED") then {
                _waypointVars set ["speed", waypointSpeed _x];
            };
            if (waypointBehaviour _x isNotEqualTo "UNCHANGED") then {
                _waypointVars set ["behaviour", waypointBehaviour _x];
            };
            if (waypointFormation _x isNotEqualTo "NO CHANGE") then {
                _waypointVars set ["formation", waypointFormation _x];
            };
            if (waypointCombatMode _x isNotEqualTo "NO CHANGE") then {
                _waypointVars set ["combat", waypointCombatMode _x];
            };
            if (waypointType _x isNotEqualTo "MOVE") then {
                _waypointVars set ["type", waypointType _x];
            };
            if (waypointScript _x isNotEqualTo "") then {
                _waypointVars set ["script", waypointScript _x];
            };
            if (waypointStatements _x isNotEqualTo ["true",""]) then {
                _waypointVars set ["statements", waypointStatements _x];
            };
            if (waypointTimeout _x isNotEqualTo [0,0,0]) then {
                _waypointVars set ["timeout", waypointTimeout _x];
            };
            if (waypointHousePosition _x isNotEqualTo -1) then {
                _waypointVars set ["house", waypointHousePosition _x];
            };
            if (waypointLoiterRadius _x isNotEqualTo -1) then {
                _waypointVars set ["loiterRadius", waypointLoiterRadius _x];
            };
            if (waypointLoiterAltitude _x isNotEqualTo -1) then {
                _waypointVars set ["loiterAltitude", waypointLoiterAltitude _x];
            };
            if (waypointLoiterType _x isNotEqualTo "CIRCLE") then {
                _waypointVars set ["loiterType", waypointLoiterType _x];
            };
            _points pushBack [
                waypointPosition _x,
                _waypointVars
            ];
        };
    } forEach _waypoints;
    if (_points isNotEqualTo []) then {
        _state set ["wp", _points];
        _state set ["current_wp", currentWaypoint _group];
    };
};

_state
