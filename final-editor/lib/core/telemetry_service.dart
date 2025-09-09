import 'dart:convert';
import 'package:web/web.dart' as web;

/// Optional telemetry service for production monitoring
/// Users can opt-out via settings
class TelemetryService {
  static TelemetryService? _instance;
  static TelemetryService get instance => _instance ??= TelemetryService._();
  
  bool _enabled = false;
  bool _initialized = false;
  
  TelemetryService._();
  
  /// Initialize telemetry service
  Future<void> initialize({bool enabled = false}) async {
    _enabled = enabled;
    _initialized = true;
    
    if (_enabled) {
      print('📊 Telemetry enabled (anonymous)');
    }
  }
  
  /// Enable or disable telemetry
  void setEnabled(bool enabled) {
    _enabled = enabled;
    if (_enabled) {
      _trackEvent('telemetry_enabled');
    }
  }
  
  /// Track parsing performance
  void trackParseTime({
    required String operation, // 'md_to_json' | 'json_to_md' | 'canonicalize'
    required int durationMs,
    required int inputSize,
  }) {
    if (!_shouldTrack()) return;
    
    _trackEvent('parse_performance', data: {
      'operation': operation,
      'duration_ms': durationMs,
      'input_size': inputSize,
      'performance_bucket': _getPerformanceBucket(durationMs, inputSize),
    });
  }
  
  /// Track anonymized errors
  void trackError({
    required String operation,
    required String errorType,
    String? context,
  }) {
    if (!_shouldTrack()) return;
    
    // Anonymize error message - remove user data
    final anonymizedContext = context != null 
      ? _anonymizeString(context)
      : null;
    
    _trackEvent('parse_error', data: {
      'operation': operation,
      'error_type': errorType,
      'context': anonymizedContext,
      'timestamp': DateTime.now().toIso8601String(),
    });
  }
  
  /// Track feature usage
  void trackFeatureUsage(String feature) {
    if (!_shouldTrack()) return;
    
    _trackEvent('feature_usage', data: {
      'feature': feature,
      'timestamp': DateTime.now().toIso8601String(),
    });
  }
  
  /// Track document statistics (anonymized)
  void trackDocumentStats({
    required int elements,
    required int characters,
    required int words,
  }) {
    if (!_shouldTrack()) return;
    
    _trackEvent('document_stats', data: {
      'elements_bucket': _getSizeBucket(elements, [1, 10, 50, 100]),
      'characters_bucket': _getSizeBucket(characters, [100, 1000, 5000, 10000]),
      'words_bucket': _getSizeBucket(words, [50, 500, 2000, 5000]),
    });
  }
  
  /// Track session information
  void trackSession({
    required String sessionId,
    required int durationMinutes,
  }) {
    if (!_shouldTrack()) return;
    
    _trackEvent('session_end', data: {
      'session_id': sessionId,
      'duration_minutes': durationMinutes,
      'duration_bucket': _getSizeBucket(durationMinutes, [1, 5, 15, 30, 60]),
    });
  }
  
  /// Check if we should track events
  bool _shouldTrack() {
    return _initialized && _enabled;
  }
  
  /// Track an event with data
  void _trackEvent(String event, {Map<String, dynamic>? data}) {
    try {
      final payload = {
        'event': event,
        'timestamp': DateTime.now().toIso8601String(),
        'session': _getSessionId(),
        'version': '1.3.0',
        'platform': 'web',
        'user_agent': _getUserAgent(),
        if (data != null) 'data': data,
      };
      
      // In production, send to analytics endpoint
      // For now, just log for debugging
      print('📊 Telemetry: ${jsonEncode(payload)}');
      
      // Send to analytics service (opt-in)
      _sendToAnalytics(payload);
      
    } catch (e) {
      // Never let telemetry break the app
      print('Telemetry error (ignored): $e');
    }
  }
  
  /// Send telemetry data to analytics service
  void _sendToAnalytics(Map<String, dynamic> payload) {
    try {
      // Only send if user has opted in
      if (!_enabled) return;
      
      // Use navigator.sendBeacon for reliable delivery
      final data = jsonEncode(payload);
      final blob = web.Blob([data].toJS);
      
      // Mock endpoint - replace with actual analytics service
      const endpoint = 'https://api.example.com/telemetry';
      
      if (web.window.navigator.has('sendBeacon')) {
        // Use sendBeacon for better reliability
        final beacon = web.window.navigator as dynamic;
        beacon.sendBeacon(endpoint, blob);
      } else {
        // Fallback to fetch
        web.window.fetch(endpoint.toJS, web.RequestInit(
          method: 'POST',
          body: blob,
          headers: {
            'Content-Type': 'application/json',
          }.toJS,
        ));
      }
    } catch (e) {
      // Silent failure - never break the app
    }
  }
  
  /// Anonymize sensitive strings
  String _anonymizeString(String input) {
    // Remove potential user data
    return input
        .replaceAll(RegExp(r'\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b'), '[EMAIL]')
        .replaceAll(RegExp(r'\b\d{3}-\d{3}-\d{4}\b'), '[PHONE]')
        .replaceAll(RegExp(r'\b\d{1,5}\s\w+\s(?:Street|St|Avenue|Ave|Road|Rd|Lane|Ln)\b'), '[ADDRESS]')
        .substring(0, 100); // Limit length
  }
  
  /// Get performance bucket for timing
  String _getPerformanceBucket(int durationMs, int inputSize) {
    final ratio = durationMs / inputSize;
    
    if (ratio < 0.01) return 'fast';
    if (ratio < 0.1) return 'normal';
    if (ratio < 1.0) return 'slow';
    return 'very_slow';
  }
  
  /// Get size bucket for metrics
  String _getSizeBucket(int value, List<int> thresholds) {
    for (int i = 0; i < thresholds.length; i++) {
      if (value <= thresholds[i]) {
        return 'bucket_$i';
      }
    }
    return 'bucket_${thresholds.length}';
  }
  
  /// Get anonymous session ID
  String _getSessionId() {
    // Use sessionStorage to maintain session ID
    try {
      var sessionId = web.window.sessionStorage['editor_session_id'];
      if (sessionId == null) {
        sessionId = _generateSessionId();
        web.window.sessionStorage['editor_session_id'] = sessionId;
      }
      return sessionId;
    } catch (e) {
      return _generateSessionId();
    }
  }
  
  /// Generate anonymous session ID
  String _generateSessionId() {
    final now = DateTime.now().millisecondsSinceEpoch;
    final random = (now * 1000 + (web.window.performance.now() * 1000).round()) % 999999;
    return 'session_${now.toString().substring(7)}_$random';
  }
  
  /// Get anonymized user agent
  String _getUserAgent() {
    try {
      final ua = web.window.navigator.userAgent;
      
      // Extract browser and version only
      if (ua.contains('Chrome')) {
        final match = RegExp(r'Chrome/(\d+)').firstMatch(ua);
        return 'Chrome/${match?.group(1) ?? 'unknown'}';
      } else if (ua.contains('Firefox')) {
        final match = RegExp(r'Firefox/(\d+)').firstMatch(ua);
        return 'Firefox/${match?.group(1) ?? 'unknown'}';
      } else if (ua.contains('Safari')) {
        final match = RegExp(r'Version/(\d+).*Safari').firstMatch(ua);
        return 'Safari/${match?.group(1) ?? 'unknown'}';
      } else if (ua.contains('Edge')) {
        final match = RegExp(r'Edge/(\d+)').firstMatch(ua);
        return 'Edge/${match?.group(1) ?? 'unknown'}';
      }
      
      return 'Unknown';
    } catch (e) {
      return 'Unknown';
    }
  }
}