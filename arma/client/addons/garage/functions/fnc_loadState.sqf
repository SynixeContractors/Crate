#include "script_component.hpp"

params ["_vehicle", "_state"];

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
    clearitemCargoGlobal _container;
    {
        _container addItemCargoGlobal [_x, _standard select 2 select 1 select _forEachIndex];
    } forEach ((_standard select 2) select 0);

    // Nested Cargo
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
        case "tex": {
            {
                _vehicle setObjectTextureGlobal [_forEachIndex, _x];
            } forEach _y;
        };
        case "hits": {
            {
                _vehicle setHitPointDamage [_x select 0, _x select 1, false];
            } foreach _y;
        };
        case "fuel": {
            _vehicle setFuel _y;
        };
        case "fuelCargo": {
            _vehicle setFuelCargo _y;
        };
        case "ammoCargo": {
            _obj setAmmoCargo _value;
        };
        case "repairCargo": {
            _obj setRepairCargo _value;
        };
        case "inventory": {
            [_obj, _value] call _fnc_addCargoForContainer;
        };
        case "turret": {
            {
                _vehicle addMagazineTurret _x;
            } forEach _y;
        };
        case "cargo": {
            {
                _x params ["_type", "_data"];
                switch (_type) do {
                    case 0: {
                        [_data, _vehicle] call ace_cargo_fnc_addCargoItem;
                    };
                    case 1: {
                        _data params ["_class", "_state"];
                        private _cargoVehicle = _class createVehicle [0,0,0];
                        [_cargoVehicle, _state] call FUNC(loadState);
                        [_cargoVehicle, _vehicle, true] call ace_cargo_fnc_loadItem;
                    };
                }
            } forEach _y;
        };
    };
} forEach _state;
