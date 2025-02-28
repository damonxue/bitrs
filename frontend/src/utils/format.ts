/**
 * 格式化数字，包括价格、数量等
 * @param value 要格式化的数值
 * @param decimals 小数位数
 * @param options 格式化选项
 * @returns 格式化后的字符串
 */
export function formatNumber(
  value: number,
  decimals: number = 2,
  options: {
    useGrouping?: boolean;
    compact?: boolean;
  } = {}
): string {
  const { useGrouping = true, compact = false } = options;

  if (value === undefined || value === null || isNaN(value)) {
    return '-';
  }

  // 对于非常小的数值，避免显示科学计数法
  const absValue = Math.abs(value);
  if (absValue > 0 && absValue < 0.000001 && !compact) {
    return '< 0.000001';
  }

  // 创建格式化选项
  const formatOptions: Intl.NumberFormatOptions = {
    minimumFractionDigits: decimals,
    maximumFractionDigits: decimals,
    useGrouping,
  };

  // 如果需要紧凑格式（例如1.2K, 1.5M）
  if (compact && absValue >= 1000) {
    formatOptions.notation = 'compact';
  }

  return new Intl.NumberFormat('zh-CN', formatOptions).format(value);
}

/**
 * 格式化时间戳为本地日期时间字符串
 * @param timestamp UNIX时间戳（秒）
 * @param formatType 格式类型
 * @returns 格式化后的日期时间字符串
 */
export function formatTimestamp(
  timestamp: number,
  formatType: 'date' | 'time' | 'datetime' = 'datetime'
): string {
  if (!timestamp) return '-';

  // 将时间戳转换为毫秒（如果是秒为单位）
  const date = new Date(timestamp * 1000);

  const options: Intl.DateTimeFormatOptions = {};

  switch (formatType) {
    case 'date':
      options.year = 'numeric';
      options.month = '2-digit';
      options.day = '2-digit';
      break;
    case 'time':
      options.hour = '2-digit';
      options.minute = '2-digit';
      options.second = '2-digit';
      break;
    case 'datetime':
    default:
      options.year = 'numeric';
      options.month = '2-digit';
      options.day = '2-digit';
      options.hour = '2-digit';
      options.minute = '2-digit';
      options.second = '2-digit';
      break;
  }

  return new Intl.DateTimeFormat('zh-CN', options).format(date);
}

/**
 * 截断地址显示，用于钱包地址等长字符串
 * @param address 完整地址
 * @param prefixLength 前缀长度
 * @param suffixLength 后缀长度
 * @returns 截断后的地址字符串
 */
export function shortenAddress(address: string, prefixLength = 4, suffixLength = 4): string {
  if (!address) return '';
  if (address.length <= prefixLength + suffixLength) return address;
  
  return `${address.slice(0, prefixLength)}...${address.slice(-suffixLength)}`;
}