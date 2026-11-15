private _strs = _this;

if (_strs isEqualType "") exitWith {
    parseNumber _strs
};

private _floats = [];
{
    if (_x isEqualType []) then {
        _floats pushBack (_x call EFUNC(common,strToFloats));
    } else {
        _floats pushBack parseNumber _x;
    };
} forEach _strs;

_floats
