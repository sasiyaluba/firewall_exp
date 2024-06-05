# 将数组转换为16进制数
def convert_array_to_hex(array):
    return ''.join([hex(x)[2:] for x in array])


array = convert_array_to_hex([
    127, 127, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 96, 0, 82, 127, 255, 96, 0, 82, 96, 32, 96, 0, 243, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 96, 32, 82, 96,
    41, 96, 0, 243, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
])

print(array)
