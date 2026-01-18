#include "..\script_component.hpp"

params ["_object", "_state", ["_ace_cargo", true]];

// Clear existing state
// Inventory
if !(_object isKindOf "GroundWeaponHolder") then {
    clearMagazineCargoGlobal _object;
    clearWeaponCargoGlobal _object;
    clearItemCargoGlobal _object;
    clearBackpackCargoGlobal _object;
    // Magazines
    {
        _x params ["_mag", "_pos"];
        _object removeMagazineTurret [_mag, _pos];
    } forEach magazinesAllTurrets _object;
    // Clear ACE Cargo
    private _loaded = _object getVariable ["ace_cargo_loaded", []];
    if (_loaded isNotEqualTo []) then {
        {
            if (_x isEqualType objNull) then {
                detach _x;
                deleteVehicle _x;
            };
        } forEach _loaded;
    };
    _object setVariable ["ace_cargo_loaded", [], true];
};

{
    switch (_x) do {
        case "dead": {
            _object setDamage [1, false];
        };
        case "tex": {
            {
                _object setObjectTextureGlobal [_forEachIndex, _x];
            } forEach _y;
        };
        case "hits": {
            {
                _object setHitPointDamage [_x select 0, _x select 1, false];
            } forEach _y;
        };
        case "fuel": {
            _object setFuel _y;
        };
        case "fuelCargo": {
            _object setFuelCargo _y;
        };
        case "plate": {
            _object setPlateNumber _y;
        };
        case "ammoCargo": {
            _object setAmmoCargo _y;
        };
        case "repairCargo": {
            _object setRepairCargo _y;
        };
        case "inventory": {
            call FUNC(addInventory);
        };
        case "turret": {
            {
                _object addMagazineTurret _x;
            } forEach _y;
        };
        case "cargo": {
            if (!_ace_cargo) then { continue };
            {
                _x params ["_type", "_data"];
                switch (_type) do {
                    case 0: {
                        [_data, _object] call ace_cargo_fnc_addCargoItem;
                    };
                    case 1: {
                        _data params ["_class", "_state"];
                        private _cargoVehicle = _class createVehicle [0,0,0];
                        [_cargoVehicle, _state] call FUNC(objectState_load);
                        [_cargoVehicle, _object, true] call ace_cargo_fnc_loadItem;
                    };
                }
            } forEach _y;
        };
        case "handcuffed": {
            [_object, _y] call ace_captives_fnc_setHandcuffed;
        };
        case "phases": {
            {
                _object animateSource [_x#0, _x#1, true];
            } forEach _y;
        };
        case "terrain": {
            hideObjectGlobal  (objectFromNetId _y);
            _object setVariable [QGVAR(terrain), _y];
        };
        case "locked": {
            _object lock _y;
        };
        case "ace_magazine": {
            _object setVariable ["ace_rearm_magazineClass", _y];
        };
        case "inflamed": {
            _object inflame _y;
        };
        case "engine": {
            _object engineOn _y;
        };
        case "light": {
            _object setPilotLight _y;
        };
        case "collision": {
            _object setCollisionLight _y;
        };
    };
} forEach _state;

// must be done after cargo is loaded
if ("cargo_space" in _state) then {
    _object setVariable ["ace_cargo_space", _state get "cargo_space"];
};
