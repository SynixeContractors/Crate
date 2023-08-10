#include "script_component.hpp"

params ["_id", "_plate", "_class", "_state"];

if (count allPlayers == 0) exitWith {
    EXTCALL("garage:spawn",[ARR_2(_id,"NoPlayers")]);
};

if (getNumber (missionConfigFile >> "synixe_template") < 3) then {
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
    private _state = createHashMapFromArray _state;
    [{
        _this call EFUNC(common,objectState_load);
    }, [_vehicle, _state]] call CBA_fnc_execNextFrame;

    EXTCALL("garage:spawn",[ARR_2(_id,"Yes")]);

    GVAR(spawned) set [_plate, _vehicle];
} else {
    private _vehicle = _class createVehicleLocal [0,0,0];
    _vehicle enableSimulation false;
    private _objType = switch (true) do {
        case (_vehicle isKindOf "Ship"): { QGVAR(sea_large) };
        case (_vehicle isKindOf "Helicopter"): { QGVAR(heli_large) };
        case (_vehicle isKindOf "Plane"): { QGVAR(plane_large) };
        case (_vehicle isKindOf "Thing"): { QGVAR(thing_medium) };
        default { QGVAR(land_large) };
    };
    deleteVehicle _vehicle;

    private _spawns = allMissionObjects _objType;

    if (count _spawns == 0) exitWith {
        EXTCALL("garage:spawn",[ARR_2(_id,"NoSpawnArea")]);
    };

    private _spawn = spawns findIf {
        private _size = getNumber (configFile >> "CfgVehicles" >> _objType >> QGVAR(size));
        count nearestObjects [_spawnPos, ["Land", "Air", "Ship", "Thing"], _size] > 0
    };
    if (_spawn == -1) exitWith {
        EXTCALL("garage:spawn",[ARR_2(_id,"AreaBlocked")]);
    };
    private _spawn = _spawns select _spawn;

    private _spawnPos = getPos _spawn;
    private _spawnDir = getDir _spawn;

    // Spawn the vehicle
    private _vehicle = _class createVehicle _spawnPos;
    _vehicle setDir _spawnDir;
    _vehicle setVariable [QGVAR(plate), _plate, true];
    _vehicle setPlateNumber _plate;
    private _state = createHashMapFromArray _state;
    [{
        _this call EFUNC(common,objectState_load);
    }, [_vehicle, _state]] call CBA_fnc_execNextFrame;

    EXTCALL("garage:spawn",[ARR_2(_id,"Yes")]);

    GVAR(spawned) set [_plate, _vehicle];
};
