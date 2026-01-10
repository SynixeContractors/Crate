#include "script_component.hpp"

params ["_id", "_plate", "_class", "_state"];

if !(isServer) exitWith {};

if (allPlayers isEqualTo []) exitWith {
    EXTCALL("garage:spawn",[ARR_2(_id,"NoPlayers")]);
};

private _vehicle = if (getNumber (missionConfigFile >> "synixe_template") < 3) then {
    private _vehicle = _class createVehicleLocal [0,0,0];
    _vehicle enableSimulation false;
    private _spawn = switch (true) do {
        case (_vehicle isKindOf "Ship"): { "spawn_sea" };
        case (_vehicle isKindOf "Helicopter"): { "spawn_heli" };
        case (_vehicle isKindOf "Plane"): { "spawn_plane" };
        case (_vehicle isKindOf "Thing"): { "spawn_thing" };
        default { "spawn_land" };
    };
    deleteVehicle _vehicle;

    private _spawnPos = markerPos _spawn;

    if (_spawnPos isEqualTo [0,0,0]) exitWith {
        EXTCALL("garage:spawn",[ARR_2(_id,"NoSpawnArea")]);
        objNull
    };

    // Check for any obstruction in the spawn area
    if (nearestObjects [_spawnPos, ["Land", "Air", "Ship", "Thing"], 5] isNotEqualTo []) exitWith {
        EXTCALL("garage:spawn",[ARR_2(_id,"AreaBlocked")]);
        objNull
    };

    // Spawn the vehicle
    private _vehicle = _class createVehicle _spawnPos;
    _vehicle setDir (markerDir _spawn);
    _vehicle
} else {
    private _vehicle = _class createVehicleLocal [0,0,0];
    _vehicle enableSimulation false;
    private _objType = switch (true) do {
        case (_vehicle isKindOf "Ship"): { "sea" };
        case (_vehicle isKindOf "Helicopter"): { "heli" };
        case (_vehicle isKindOf "Plane"): { "plane" };
        case (_vehicle isKindOf "Thing"): { "thing" };
        default { "land" };
    };
    private _bounds = boundingBoxReal _vehicle;
    private _objSize = (_bounds#0 distance _bounds#1) / 2;
    deleteVehicle _vehicle;

    private _spawns = [];
    {
        if (_objType in _x) then {
            _spawns append allMissionObjects _x;
        };
    } forEach SPAWN_TYPES;

    private _spawns = _spawns select {
        getNumber (configOf _x >> QGVAR(size)) > _objSize
    };

    if (_spawns isEqualTo []) exitWith {
        EXTCALL("garage:spawn",[ARR_2(_id,"NoSpawnArea")]);
        objNull
    };

    private _spawn = _spawns findIf {
        nearestObjects [getPos _x, ["Land", "Air", "Ship", "Thing"], _objSize + 0.5] isEqualTo []
    };
    if (_spawn == -1) exitWith {
        EXTCALL("garage:spawn",[ARR_2(_id,"AreaBlocked")]);
        objNull
    };
    private _spawn = _spawns select _spawn;

    private _vehicle = _class createVehicle (getPos _spawn);
    _vehicle setDir (getDir _spawn);
    _vehicle
};

if (_vehicle isEqualTo objNull) exitWith {};

EXTCALL("garage:spawn",[ARR_2(_id,"Yes")]);

_vehicle setVariable [QGVAR(plate), _plate, true];
_vehicle setPlateNumber _plate;
_vehicle setVariable ["ace_tagging_canTag", false, true];
_vehicle setVariable ["crate", true, true];
[{
    _this call EFUNC(common,objectState_load);
}, [_vehicle, createHashMapFromArray _state]] call CBA_fnc_execNextFrame;

GVAR(spawned) set [_plate, _vehicle];

// this may still be having issues?
// do it last just in case
[_vehicle, _plate, 0.4, "ffd731"] call ace_tagging_fnc_stencilVehicle;

if ("tex_source" in _state) then {
    private _config = configOf _vehicle;

    private _textureList = getArray (_config >> "textureList");
    if (_textureList isEqualTo []) exitWith {true};

    private _textureSource = _config >> "TextureSources" >> (_state get "tex_source");
    private _textures = getArray (_textureSource >> "textures");
    private _materials = getArray (_textureSource >> "materials");

    {
        _vehicle setObjectTextureGlobal [_forEachIndex, _x];
    } forEach _textures;

    {
        _vehicle setObjectMaterialGlobal [_forEachIndex, _x];
    } forEach _materials;
};
