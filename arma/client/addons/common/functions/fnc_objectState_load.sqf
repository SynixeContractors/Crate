#include "script_component.hpp"

params ["_object", "_state", ["_ace_cargo", true]];

// Clear existing state
// Inventory
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
[_object] call ace_cargo_fnc_validateCargoSpace;

private _fnc_addCargoForContainer = {
    params ["_container", "_cargo"];

    private _standard = _cargo select 0;
    private _nested = _cargo select 1;

    // Magazine cargo
    clearMagazineCargoGlobal _container;
    {
        _container addMagazineCargoGlobal [_x, _standard select 0 select 1 select _forEachIndex];
    } forEach ((_standard select 0) select 0);

    // Weapon Cargo
    clearWeaponCargoGlobal _container;
    {
        _container addWeaponWithAttachmentsCargoGlobal [_x, 1];
    } forEach (_standard select 1);

    // Item Cargo
    clearItemCargoGlobal _container;
    {
        _container addItemCargoGlobal [_x, _standard select 2 select 1 select _forEachIndex];
    } forEach ((_standard select 2) select 0);

    // Nested Cargo
    clearBackpackCargoGlobal _container;
    {
        _x params ["_class", "_items"];
        _container addBackpackCargoGlobal [_class, 1];
        // Thanks Bohemia, the only way to find the container I just added is to loop over every container...
        {
            private _found = false;
            if (_x select 0 == _class) then {
                // More Elegant code
                if (
                    !_found &&
                    { (_x select 1) getVariable [QGVAR(NOEDIT), true] }
                ) then {
                    [_x select 1, (_items)] call _fnc_addCargoForContainer;
                    (_x select 1) setVariable [QGVAR(NOEDIT), false];
                    _found = true;
                };
            };
        } forEach everyContainer _container;
    } forEach _nested;
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
            [_object, _y] call _fnc_addCargoForContainer;
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
                        [_cargoVehicle, _state] call FUNC(loadState);
                        [_cargoVehicle, _object, true] call ace_cargo_fnc_loadItem;
                    };
                }
            } forEach _y;
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
