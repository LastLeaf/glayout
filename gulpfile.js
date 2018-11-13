var fs = require('fs-extra')
var process = require('process')
var execFile = require('child_process').execFile
var gulp = require('gulp')
var concat = require('gulp-concat')
var sourcemaps = require('gulp-sourcemaps')
var footer = require('gulp-footer')
var webpack = require('webpack')

var libCompilerConfig = require('./asmjs/webpack.config')

var execCommand = function(cmd, args, cwd, cb) {
  var cp = execFile(cmd, args, {cwd: cwd}, function(err, stdout, stderr) {
    if (err) {
      cb(err)
      return
    }
    cb()
  })
  cp.stdout.pipe(process.stdout)
  cp.stderr.pipe(process.stderr)
}

var convertSourceMapPath = function() {
  return sourcemaps.mapSources(function(sourcePath, file) {
    if (sourcePath.match(/^webpack:\/\/__glayoutLib__\/webpack\//)) {
      return sourcePath.replace('webpack://__glayoutLib__/webpack/', '__internal__/webpack/')
    }
    if (sourcePath.match(/^webpack:\/\/__glayoutLib__\/asmjs\//)) {
      return sourcePath.replace('webpack://__glayoutLib__/asmjs/', 'asmjs/')
    }
    if (sourcePath.match(/^..\//)) {
      return sourcePath.replace(/^(..\/)+/, '__internal__/')
    }
    if (sourcePath.match(/^src\//)) {
      return sourcePath
    }
    return '__internal__/' + sourcePath
  })
}

var toUnderlineCase = function(str) {
  return str.replace(/[A-Z]+/g, function(c) {
    return '_' + c.toLowerCase()
  })
}

var generateLibInterfaces = function(path) {
  var slices = ['mergeInto(LibraryManager.library, {']
  // eslint-disable-next-line
  var lib = (new Function(fs.readFileSync(path, {encoding: 'utf8'}) + '\n; return __glayoutLib__'))()
  for (var k in lib) {
    var func = lib[k]
    if (typeof func === 'function') {
      var argsLength = func.length
      var funcName = k
      var argList = ''
      for (var i = 0; i < argsLength; i++) {
        if (i) argList += ', '
        argList += 'arg' + i
      }
      slices.push(toUnderlineCase(funcName) + ': function(' + argList + ') { return window.__glayoutLib__.' + funcName + '(' + argList + ') },')
    }
  }
  slices.push('})')
  return slices.join('\n')
}

gulp.task('clean-asm', function(cb) {
  execCommand('cargo', ['clean'], '.', cb)
})

gulp.task('clean-lib', function(cb) {
  fs.remove('./asmjs/bin', cb)
})

gulp.task('clean', ['clean-asm', 'clean-lib'], function(cb) {
  fs.remove('./bin', cb)
})

gulp.task('compile-lib-debug', function(cb) {
  webpack(libCompilerConfig[0]).run(cb)
});

gulp.task('generate-lib-interfaces-debug', ['compile-lib-debug'], function(cb) {
  fs.writeFile('./asmjs/bin/interfaces.js', generateLibInterfaces('./asmjs/bin/glayout-lib.js'), cb)
})

gulp.task('compile-lib-release', function(cb) {
  webpack(libCompilerConfig[1]).run(cb)
});

gulp.task('generate-lib-interfaces-release', ['compile-lib-release'], function(cb) {
  fs.writeFile('./asmjs/bin/interfaces.js', generateLibInterfaces('./asmjs/bin/glayout-lib.min.js'), cb)
})

gulp.task('compile-asm-debug', ['generate-lib-interfaces-debug'], function(cb) {
  execCommand('cargo', ['build', '--target=asmjs-unknown-emscripten'], '.', cb)
})

gulp.task('compile-asm-release', ['generate-lib-interfaces-release'], function(cb) {
  execCommand('cargo', ['build', '--target=asmjs-unknown-emscripten', '--release'], '.', cb)
})

gulp.task('build-release', ['compile-asm-release'], function(cb) {
  return gulp.src(['./asmjs/bin/glayout-lib.min.js', './target/asmjs-unknown-emscripten/release/glayout-main.js'])
    .pipe(concat('glayout.min.js'))
    .pipe(footer(';window.glayout=__glayoutLib__.extern;'))
    .pipe(gulp.dest('./bin/'))
})

gulp.task('build-debug-with-sourcemap', ['compile-asm-debug'], function(cb) {
  return gulp.src(['./asmjs/bin/glayout-lib.js', './target/asmjs-unknown-emscripten/debug/deps/glayout_main-*.js'])
    .pipe(sourcemaps.init({loadMaps: true}))
    .pipe(concat('glayout.js'))
    .pipe(convertSourceMapPath())
    .pipe(footer(';window.glayout=__glayoutLib__.extern;'))
    .pipe(sourcemaps.write('.', {includeContent: true}))
    .pipe(gulp.dest('./bin/'))
})

gulp.task('build-debug', ['compile-asm-debug'], function(cb) {
  return gulp.src(['./asmjs/bin/glayout-lib.js', './target/asmjs-unknown-emscripten/debug/deps/glayout_main-*.js'])
    .pipe(sourcemaps.init({loadMaps: true}))
    .pipe(concat('glayout.js'))
    .pipe(convertSourceMapPath())
    .pipe(footer(';window.glayout=__glayoutLib__.extern;'))
    .pipe(sourcemaps.write('.', {includeContent: false, sourceRoot: '..'}))
    .pipe(gulp.dest('./bin/'))
})

gulp.task('watch', ['build-debug'], function() {
  gulp.watch(['src/*.rs', 'src/**/*.rs', 'asmjs/*.js',  'asmjs/src/*.js', 'asmjs/src/**/*.js', 'src/**/*.glsl'], ['build-debug'])
    .on('change', function(event) {
      console.log('file: ' + event.path + ' was ' + event.type);
    })
})

gulp.task('default', ['build-release'])
