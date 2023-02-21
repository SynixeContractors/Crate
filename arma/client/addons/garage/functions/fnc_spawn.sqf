#include "script_component.hpp"

params ["_id", "_plate", "_class", "_state"];

if (count allPlayers == 0) exitWith {
    EXTCALL("garage:spawn",[ARR_2(_id,"NoPlayers")]);
};

private _vehicle = _class createVehicleLocal [0,0,0];
_vehicle enableSimulation false;
private _type = switch (true) do {
    case (_vehicle isKindOf "Ship"): { "sea" };
    case (_vehicle isKindOf "Helicopter"): { "heli" };
    case (_vehicle isKindOf "Plane"): { "plane" };
    case (_vehicle isKindOf "Thing"): { "thing" };
    default { "land" };
};
deleteVehicle _vehicle;

private _spawn = switch (_type) do {
    case "sea": { "spawn_sea" };
    case "heli": { "spawn_heli" };
    case "plane": { "spawn_plane" };
    case "thing": { "spawn_thing" };
    default { "spawn_land" };
};
private _spawnPos = markerPos _spawn;
private _spawnDir = markerDir _spawn;

if (_spawnPos isEqualTo [0,0,0]) exitWith {
    EXTCALL("garage:spawn",[ARR_2(_id,"NoSpawnArea")]);
};

// Check for any obstruction in the spawn area
if (count nearestObjects [_spawnPos, ["Land", "Air", "Ship", "Thing"], SPAWN_SIZE] > 0) exitWith {
    EXTCALL("garage:spawn",[ARR_2(_id,"AreaBlocked")]);
};

// Spawn the vehicle
private _vehicle = _class createVehicle _spawnPos;
_vehicle setDir _spawnDir;
_vehicle setVariable [QGVAR(plate), _plate, true];
_vehicle setPlateNumber _plate;
[{
    params ["_vehicle", "_state"];
    // Clear Inventory
    clearMagazineCargoGlobal _vehicle;
    clearWeaponCargoGlobal _vehicle;
    clearitemCargoGlobal _vehicle;
    clearBackpackCargoGlobal _vehicle;
    // Clear Magazines
    {
        _x params ["_mag", "_pos"];
        _vehicle removeMagazineTurret [_mag, _pos];
    } forEach magazinesAllTurrets _vehicle;
    // Clear ACE Cargo
    private _loaded = _vehicle getVariable ["ace_cargo_loaded", []];
    if (_loaded isNotEqualTo []) then {
        {
            if (_x isEqualType objNull) then {
                detach _x;
                deleteVehicle _x;
            };
        } forEach _loaded;
    };
    _vehicle setVariable ["ace_cargo_loaded", [], true];
    [_vehicle] call ace_cargo_fnc_validateCargoSpace;
    [_vehicle, _state] call FUNC(loadState);
}, [_vehicle, _state]] call CBA_fnc_execNextFrame;

EXTCALL("garage:spawn",[ARR_2(_id,"Yes")]);

GVAR(spawned) set [_plate, _vehicle];
