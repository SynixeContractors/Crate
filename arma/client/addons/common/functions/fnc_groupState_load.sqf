#include "script_component.hpp"

params ["_group", "_state"];

{
    switch (_x) do {
        case "id": {
            _group setGroupIdGlobal _y;
        };
        case "combat": {
            _group setCombatMode _y;
        };
        case "behaviour": {
            _group setBehaviour _y;
        };
        case "speed": {
            _group setSpeedMode _y;
        };
        case "formation": {
            _group setFormation _y;
        };
        case "wp": {
            {
                _x params ["_pos", "_vars"];
                private _wp = _group addWaypoint [_pos, 0];
                {
                    switch (_x) do {
                        case "visible": {
                            _wp setWaypointVisible _y;
                        };
                        case "description": {
                            _wp setWaypointDescription _y;
                        };
                        case "speed": {
                            _wp setWaypointSpeed _y;
                        };
                        case "behaviour": {
                            _wp setWaypointBehaviour _y;
                        };
                        case "formation": {
                            _wp setWaypointFormation _y;
                        };
                        case "combat": {
                            _wp setWaypointCombatMode _y;
                        };
                        case "type": {
                            _wp setWaypointType _y;
                        };
                        case "script": {
                            _wp setWaypointScript _y;
                        };
                        case "statements": {
                            _wp setWaypointStatements _y;
                        };
                        case "timeout": {
                            _wp setWaypointTimeout _y;
                        };
                        case "house": {
                            _wp setWaypointHousePosition _y;
                        };
                        case "loiterRadius": {
                            _wp setWaypointLoiterRadius _y;
                        };
                        case "loiterAltitude": {
                            _wp setWaypointLoiterAltitude _y;
                        };
                        case "loiterType": {
                            _wp setWaypointLoiterType _y;
                        };
                    };
                } forEach _vars;
            } forEach _y;
        };
        case "current_wp": {
            _group setCurrentWaypoint _y;
        };
    };
} forEach _state;
