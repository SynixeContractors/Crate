#include "script_component.hpp"

params ["_unit", "_state"];

{
    switch (_x) do {
        case "name": {
            _unit setName _y;
        };
        case "loadout": {
            [_unit, _y] call CBA_fnc_setLoadout;
        };
        case "face": {
            _unit setFace _y;
        };
        case "speaker": {
            _unit setSpeaker _y;
        };
        case "rank": {
            _unit setRank _y;
        };
        case "pitch": {
            _unit setPitch _y;
        };
        case "alive": {
            if !(_y) then {
                _unit setDamage [1, false];
            };
        };
        case "flashlight": {
            _unit enableGunLights "ForceOn";
        };
        case "irlaser": {
            _unit enableIRLasers _y;
        };
        // case "weapon"
        case "unitPos": {
            _unit setUnitPos _y;
        };
        case "combat": {
            _unit setUnitCombatMode _y;
        };
        case "behaviour": {
            _unit setUnitBehaviour _y;
        };
        case "vehicle": {
            _y params ["_vid", "_seat"];
            private _objects = allMissionObjects "All";
            private _vehicle = _objects select (_objects findIf { (_x getVariable [QGVAR(id), "-"]) isEqualTo _vid });
            switch (_seat) do {
                case "driver": {
                    _unit moveInDriver _vehicle;
                };
                case "gunner": {
                    _unit moveInGunner _vehicle;
                };
                case "commander": {
                    _unit moveInCommander _vehicle;
                };
                default {
                    _unit moveInCargo [_vehicle, _seat];
                };
            };
        };
        case "ace_surrender": {
            [_unit, true] call ACE_captives_fnc_setSurrendered;
        };
        case "ace_handcuffed": {
            [_unit, true] call ACE_captives_fnc_setHandcuffed;
        };
    };
} forEach _state;
