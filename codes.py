# generate a list of 16 alphanumeric codes
def generate_codes():
    import random
    import string
    codes = []
    for i in range(16):
        code = ''.join(random.choice(string.ascii_lowercase + string.digits) for _ in range(16))
        codes.append(code)
    return codes

if __name__ == '__main__':
    print("\n".join(generate_codes()))
