#include "..\script_component.hpp"

private _strs = _this;

if (_strs isEqualType "") exitWith {
    parseNumber _strs
};

private _floats = [];
{
    if (_x isEqualType []) then {
        _floats pushBack (_x call EFUNC(common,strToFloats));
    } else {
        if (_x isEqualType "") then {
            _floats pushBack parseNumber _x;
        } else {
            _floats pushBack _x;
        };
    };
} forEach _strs;

_floats
